use std::os::fd;
use std::env;
use std::env::Args;
use std::net::Ipv4Addr;
use std::time::SystemTime;

use nix::sys::socket;
use nix::sys::socket::AddressFamily;
use nix::sys::socket::SockType;
use nix::sys::socket::SockFlag;
use nix::sys::socket::SockProtocol;

use internet_checksum::checksum;

// Traits
use std::os::fd::AsRawFd;

fn get_address(args: &mut Args) -> Result<socket::SockaddrIn, String> {
    let executable_name = args.next().unwrap();
    let addr = args.next().ok_or_else(|| {
        println!("Usage: {} <ipv4-address>", executable_name);
        String::from("Not enough arguments!")
    })?;

    if let Some(_) = args.next() {
        println!("Usage: {} <ipv4-address>", executable_name);
        return Err(String::from("Too many arguments!"));
    }
    let ip4address: Ipv4Addr = addr.parse().map_err(|e| {
        println!("{}", e);
        format!("Unable to parse {} as an IPv4 address!", addr)
    })?;

    let octets = ip4address.octets();
    return Ok(socket::SockaddrIn::new(octets[0], octets[1], octets[2], octets[3], 0));
}

fn ping() -> Result<(), String> {
    let address = get_address(&mut env::args())?;
    let sock: fd::OwnedFd = socket::socket(
        AddressFamily::Inet,
        SockType::Raw,
        SockFlag::empty(),
        SockProtocol::Icmp
    ).map_err(|e| { format!("{} - Unable to create socket!", e) })?;

    let mut buf: Vec<u8> = Vec::new();
    buf.resize(8, 0);
    buf[0] = 0x08; // Type = 0x08 for echo request
    buf[7] = 0x01; // Set sequence number to 1

    // Compute and set checksum
    let checksum = checksum(buf.as_slice());
    buf[2..4].copy_from_slice(&checksum);

    println!("PING {}", address.ip().to_string());
    let start_time = SystemTime::now();

    socket::sendto(
        sock.as_raw_fd(),
        buf.as_slice(),
        &address,
        socket::MsgFlags::empty()
    ).map_err(|e| { format!("{} - Unable to send ICMP message!", e) })?;

    buf.clear();
    buf.resize(1024, 0);

    let (recv_bytes, sender_addr): (usize, Option<socket::SockaddrIn>) =
        socket::recvfrom(sock.as_raw_fd(), buf.as_mut_slice())
        .map_err(|e| { format!("{} - Error trying to receive ICMP response!", e) })?;

    let end_time = SystemTime::now();
    let elapsed_duration = end_time
        .duration_since(start_time)
        .map_err(|e| format!("{} - Unable to compute elapsed time!", e))?;

    if recv_bytes >= 1024 {
        panic!("Buffer full when receiving!");
    }

    // Consume IPv4 header first
    let ip4_header_size = ((buf[0] & 0x0F) as usize) * 4;
    let response = &buf[ip4_header_size..];

    let responder = sender_addr
        .map(|sock_add| sock_add.ip().to_string())
        .unwrap_or(String::from("unknown"));


    if response[0] == 0 {
        println!("ICMP echo response received from address {} in {:.3} ms", responder,
            elapsed_duration.as_micros() as f64 / 1000 as f64);
        Ok(())
    } else {
        Err(format!("Unexpected type {} (with code {}) for ICMP response!", response[0], response[1]))
    }
}

fn main() {
    if let Err(e) = ping() {
        println!("{}", e);
        std::process::exit(1);
    }
}
