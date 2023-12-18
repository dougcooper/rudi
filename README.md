
# README

`rudi - (r)ecieve (u)dp (di)stribution`

>deduplicates receiving udp sockets and distributes a copy of the data over broadcast channels for downstream tasks 

```rust
let mut udp = UdpManager::default();
let unicast = IpConfigV4 {
    cast_mode: CastMode::Unicast,
    addr: "0.0.0.0:6993".parse::<SocketAddrV4>().unwrap(),
};
let mut rx1 = udp.subscribe(&unicast).await.unwrap();

tokio::spawn(async move {
    while let Ok(data) = rx1.recv().await {
        println!("{:?}", data);
    }
});

assert_eq!(udp.count(), 1);
```

## test using cross

```
cargo install cross
cross test --target x86_64-unknown-linux-gnu
```

