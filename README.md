
# README

`rudi - (r)ecieve (u)dp (di)stribution`

>deduplicates receiving udp sockets and distributes a copy of the data over broadcast channels for downstream tasks 

```rust
let mut udp = UdpManager::default();
let unicast = IpConfigV4 {
    cast_mode: CastMode::Unicast,
    addr: "0.0.0.0:6993".parse::<SocketAddrV4>().unwrap(),
};
let channel_size = Some(100);
let mut rx1 = udp.subscribe(&unicast,channel_size).await.unwrap();
let mut rx2 = udp.subscribe(&unicast,None).await.unwrap();

tokio::spawn(async move {
    while let Ok(data) = rx1.recv().await {
        println!("rx1 {:?}", data);
    }
});

tokio::spawn(async move {
    while let Ok(data) = rx2.recv().await {
        println!("rx2 {:?}", data);
    }
});

assert_eq!(udp.count(), 2);
```

## test using cross

```
cargo install cross
cross test --target x86_64-unknown-linux-gnu
```

## ROADMAP

- [ ] restrict to interface using connect

## Issues

- [ ] restricting to interface