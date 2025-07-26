use nix::sys::socket;
use nix::sys::socket::AddressFamily;
use nix::sys::socket::SockType;
use nix::sys::socket::SockFlag;
use nix::sys::socket::SockProtocol;

use std::os::fd;
use std::os::fd::AsRawFd;
use std::io::Write;

fn main() {
    let sock: fd::OwnedFd = socket::socket(
        AddressFamily::Inet,
        SockType::Raw,
        SockFlag::empty(),
        SockProtocol::Icmp
    ).expect("Unable to create a socket!");

    let mut buf: Vec<u8> = Vec::new();
    buf.push(0x08); // Type = 0x08 for echo request
    buf.push(0x00); // Code = 0x00 for echo request
    buf.push(0xf7); buf.push(0xfd); // Add 2 octets checksum
    buf.push(0x00); buf.push(0x01); // Zero identifier
    buf.push(0x00); buf.push(0x01); // Zero sequence number

    let _sent_bytes = socket::sendto(
        sock.as_raw_fd(),
        buf.as_slice(),
        &socket::SockaddrIn::new(10, 0, 0, 1, 0),
        socket::MsgFlags::empty()
    ).expect("Unable to send message!");

    // println!("Message of {} bytes sent, I guess!", sent_bytes);

    buf.clear();
    buf.resize(28, 0);

    let recv_result: nix::Result<(usize, Option<socket::SockaddrIn>)> =
        socket::recvfrom(sock.as_raw_fd(), buf.as_mut_slice());

    let (_recv_bytes, _) = recv_result.expect("Error receiving response!");

    // println!("Message of {} bytes received, I guess!", recv_bytes);
    let mut out = std::io::stdout();
    out.write_all(buf.as_slice()).unwrap();
    out.flush().unwrap();
}
