use std::net::UdpSocket;
use std::io;
use std::str;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use data_encoding::BASE32;

#[derive(Default,Debug, Clone)]
struct Payload {
    chunk_id: u32,
    max_chunk: u32,
    data_id: u32,
    base32_data: String,
}

fn parse_dns_query(data: &[u8]) -> Payload {

    let mut end_position= 13;
    let mut start_position  = 13;
    let mut len_of_bytes = data[12] as usize; 
    let mut count = 0;
    let mut payload = Payload::default();

    while len_of_bytes != 0 {
        end_position = start_position + len_of_bytes;

        let domain = String::from_utf8_lossy(
            &data[start_position..end_position]);

        start_position = end_position + 1;
        len_of_bytes = data[end_position] as usize;

        count+=1;
        match count {
            1 => payload.data_id = domain.to_string().parse().unwrap(),
            2 => payload.chunk_id = domain.to_string().parse().unwrap(),
            3 => payload.max_chunk = domain.to_string().parse().unwrap(),
            4 => payload.base32_data = domain.to_string(),
            _ => break
        }
    }

    payload
}

fn main() -> io::Result<()> {

    let socket = UdpSocket::bind("0.0.0.0:53")?;
    println!("UDP server listening on port 53");
    let mut buf = [0; 1024]; 

    let mut recv_payloads: HashMap<u32, Vec<Payload>> = HashMap::new();

    loop {
        let (_, _) = socket.recv_from(&mut buf)?;
        let p = parse_dns_query(&buf);
        let data_id = p.data_id.clone();
        let max_chunk = p.max_chunk.clone() as usize;

        recv_payloads.entry(data_id)
        .or_insert_with(Vec::<Payload>::new)
        .push(p);

        let p_vec: &Vec<Payload> = recv_payloads.get(&data_id).unwrap();
        if p_vec.len() == max_chunk + 1 {
            let mut sorted_vec = p_vec.clone();
            sorted_vec.sort_by_key(|p| p.chunk_id);

            let first_payload = sorted_vec.remove(0);
            
            let mut file = File::create(
                get_string(&first_payload.base32_data))
                .unwrap();
        
            for p in sorted_vec.iter() {
                file.write(get_string(&p.base32_data).as_bytes()).unwrap();
            }
        }
    
    }

}

fn get_string(nopadding_base32_string: &str) -> String {
    str::from_utf8(&BASE32.decode(
            add_padding(
    &nopadding_base32_string.to_uppercase().to_string()).as_bytes())
        .unwrap())
        .unwrap()
        .to_string()
}

fn add_padding(base32_string: &str) -> String {
    let mut padded_string = base32_string.to_string();
    while padded_string.len() % 8 != 0 {
        padded_string.push('=');
    }
    padded_string
}