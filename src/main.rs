use std::env;
use std::os::fd;
use std::env::Args;
use std::net::Ipv4Addr;
use std::time::SystemTime;

use nix::sys::socket;
use nix::sys::socket::{AddressFamily, SockType, SockFlag, SockProtocol, MsgFlags};

use internet_checksum::checksum;

use std::os::fd::AsRawFd;   // Trait

const UNREACHABLE_CODE_MSGS: [&'static str; 16] = [
    "Destination network unreachable", "Destination host unreachable", "Destination protocol unreachable",
    "Destination port unreachable", "Fragmentation required, and DF flag set", "Source route failed",
    "Destination network unknown", "Destination host unknown", "Source host isolated",
    "Network administratively prohibited", "Host administratively prohibited", "Network unreachable for ToS",
    "Host unreachable for ToS", "Communication administratively prohibited", "Host Precedence Violation",
    "Precedence cutoff in effect"
];

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
    Ok(socket::SockaddrIn::new(octets[0], octets[1], octets[2], octets[3], 0))
}

fn ping() -> Result<(), String> {
    let address = get_address(&mut env::args())?;
    let sock: fd::OwnedFd = socket::socket(
        AddressFamily::Inet, SockType::Raw, SockFlag::empty(), SockProtocol::Icmp
    ).map_err(|e| { format!("{} - Unable to create socket!", e) })?;

    // Create ICMP echo request (type 8, code 0), id 0, sequence 1
    let mut buf: Vec<u8> = vec![0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01];

    // Compute and set checksum
    let checksum = checksum(buf.as_slice());
    buf[2..4].copy_from_slice(&checksum);

    println!("PING {}", address.ip().to_string());
    let start_time = SystemTime::now();

    socket::sendto(sock.as_raw_fd(), buf.as_slice(), &address, MsgFlags::empty())
        .map_err(|e| { format!("{} - Unable to send ICMP message!", e) })?;

    buf.resize(256, 0); // An arbitrary value

    let (recv_bytes, sender_addr): (usize, Option<socket::SockaddrIn>) =
        socket::recvfrom(sock.as_raw_fd(), buf.as_mut_slice())
        .map_err(|e| { format!("{} - Error trying to receive ICMP response!", e) })?;
    if recv_bytes >= buf.len() {
        panic!("Buffer full when receiving!");
    }

    let end_time = SystemTime::now();
    let elapsed_duration = end_time
        .duration_since(start_time)
        .map_err(|e| format!("{} - Unable to compute elapsed time!", e))?;

    // For raw IP sockets, the response contains the IP header as well. The 4 LSBs of the first
    // octet is header size as the number of 32-bit words. We extract this size and look at the
    // rest of the message (the actual ICMP response header).
    let ip4_header_size = ((buf[0] & 0x0F) as usize) * 4;
    let response = &buf[ip4_header_size..];

    let responder = sender_addr
        .map(|sock_add| sock_add.ip().to_string())
        .unwrap_or(String::from("unknown"));

    if response[0] == 0 || (response[0] == 8 && address.ip().is_loopback()) {
        println!("From {}: Echo response in {:.3} ms", responder,
            elapsed_duration.as_micros() as f64 / 1000 as f64);
    } else if response[0] == 3 && response[1] < 16 {
        println!("From {}: Unreachable - {}", responder, UNREACHABLE_CODE_MSGS[response[1] as usize]);
    } else {
        return Err(format!("Unexpected type {} (with code {}) for ICMP response!", response[0], response[1]))
    }
    return Ok(())
}

fn main() {
    if let Err(e) = ping() {
        println!("{}", e);
        std::process::exit(1);
    }
}
