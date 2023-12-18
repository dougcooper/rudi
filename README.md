
# README

`rudi - (r)ecieve (u)dp (di)stribution`

>deduplicates receiving udp sockets and distributes a copy of the data over broadcast channels for downstream tasks 

see tests/exampes.rs for usage


## test using cross

```
cargo install cross
cross test --target x86_64-unknown-linux-gnu
```

