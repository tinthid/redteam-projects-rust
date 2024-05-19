#!/bin/bash
cargo build --bin $1 --release
ext=$?
if [[ $ext -ne 0 ]]; then
    exit $ext
fi

sudo setcap cap_net_raw,cap_net_admin=eip ./target/release/$1
./target/release/$1 ${@:2}
# pid=$!
# sudo ip addr add 192.168.1.123/24 dev tun0
# sudo ip link set up dev tun0
# trap "kill $pid" INT TERM
# wait $pid
