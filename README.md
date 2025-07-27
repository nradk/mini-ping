# mini-ping
_This is toy software, in the sense defined [here](https://blog.jsbarretto.com/post/software-is-joy).
It was written purely for the fun and learning._
mini-ping is an extremely minimal implementation of the `ping` utility in about 100 lines of Rust.
It sends an ICMP echo request, confirms that an ICMP echo response was received and displays the
round-trip-time in milliseconds.

## Usage
1. Build the project
```
$ cargo build
```
2. Set the `CAP_NET_RAW` capability on the binary, required to create raw sockets.
```
$ sudo setcap cap_net_raw+ep target/debug/mini-ping
```
3. Execute the binary, passing an IPv4 address.
```
$ target/debug/mini-ping 1.1.1.1
PING 1.1.1.1
From 1.1.1.1: Echo response in 28.763 ms
```
