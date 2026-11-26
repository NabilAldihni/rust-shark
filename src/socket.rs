use std::fs;
use std::io;
use std::mem::size_of;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

const ETH_P_ALL: u16 = 0x0003;
// const PACKET_OUTGOING: u8 = 4;
const IF_LOCATION: &str = "/sys/class/net/";
const IF_FILE: &str = "ifindex";

pub fn open_packet_socket(interface: &str) -> io::Result<OwnedFd> {
    let ifindex: i32 = fs::read_to_string(format!("{IF_LOCATION}{interface}/{IF_FILE}"))?
        .trim()
        .parse()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;


    let raw_fd = unsafe { libc::socket(libc::AF_PACKET, libc::SOCK_RAW, ETH_P_ALL.to_be() as i32) };

    if raw_fd < 0 {
        return Err(io::Error::last_os_error());
    }

    let fd = unsafe { OwnedFd::from_raw_fd(raw_fd) };

    let address = libc::sockaddr_ll {
        sll_family: libc::AF_PACKET as u16,
        sll_protocol: ETH_P_ALL.to_be(),
        sll_ifindex: ifindex,
        sll_hatype: 0,
        sll_pkttype: 0,
        sll_halen: 0,
        sll_addr: [0; 8],
    };

    let result = unsafe {
        libc::bind(
            fd.as_raw_fd(),
            (&address as *const libc::sockaddr_ll).cast(),
            size_of::<libc::sockaddr_ll>() as libc::socklen_t,
        )
    };

    if result < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(fd)
}

// fn mac_address(bytes: &[u8]) -> String {
//     bytes
//         .iter()
//         .map(|b| format!("{b:02x}"))
//         .collect::<Vec<_>>()
//         .join(":")
// }

// fn select_interface() -> Result<String, Box<dyn std::error::Error>> {
//     let mut interfaces = fs::read_dir("/sys/class/net")?
//         .map(|entry| entry.map(|entry| entry.file_name().to_string_lossy().into_owned()))
//         .collect::<Result<Vec<_>, _>>()?;
//     interfaces.sort();

//     if interfaces.is_empty() {
//         return Err(io::Error::new(
//             io::ErrorKind::InvalidInput,
//             "No network interfaces were found",
//         )
//         .into());
//     }

//     println!("Available network interfaces:");
//     for (index, interface) in interfaces.iter().enumerate() {
//         println!("  {}. {interface}", index + 1);
//     }

//     print!("Select an interface by number: ");
//     use std::io::Write;
//     io::stdout().flush()?;

//     let mut input = String::new();
//     io::stdin().read_line(&mut input)?;

//     let selection = input.trim().parse::<usize>().map_err(|_| {
//         io::Error::new(
//             io::ErrorKind::InvalidInput,
//             "Enter a valid interface number",
//         )
//     })?;
//     let index = selection.checked_sub(1).ok_or_else(|| {
//         io::Error::new(
//             io::ErrorKind::InvalidInput,
//             "Interface number is out of range",
//         )
//     })?;

//     interfaces.get(index).cloned().ok_or_else(|| {
//         io::Error::new(
//             io::ErrorKind::InvalidInput,
//             "Interface number is out of range",
//         )
//         .into()
//     })
// }


// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let interface = select_interface()?;
//     let fd = open_packet_socket(&interface)?;

//     println!("Listening on interface: {interface}");

//     let mut buffer = vec![0u8; 65535];

//     loop {
//         let mut sender = libc::sockaddr_ll {
//             sll_family: 0,
//             sll_protocol: 0,
//             sll_ifindex: 0,
//             sll_hatype: 0,
//             sll_pkttype: 0,
//             sll_halen: 0,
//             sll_addr: [0; 8],
//         };

//         let mut sender_len = size_of::<libc::sockaddr_ll>() as libc::socklen_t;

//         let n = unsafe {
//             libc::recvfrom(
//                 fd.as_raw_fd(),
//                 buffer.as_mut_ptr().cast::<libc::c_void>(),
//                 buffer.len(),
//                 0,
//                 (&mut sender as *mut libc::sockaddr_ll).cast(),
//                 &mut sender_len,
//             )
//         };

//         if n < 0 {
//             let error = io::Error::last_os_error();

//             if error.kind() == io::ErrorKind::Interrupted {
//                 continue;
//             }

//             return Err(error.into());
//         }

//         let n = n as usize;

//         // Ignore packets transmitted by this machine.
//         if sender.sll_pkttype == PACKET_OUTGOING {
//             continue;
//         }

//         println!("\nReceived packet: {n} bytes");

//         // Ethernet header is 14 bytes.
//         if n >= 14 {
//             let dst = mac_address(&buffer[0..6]);
//             let src = mac_address(&buffer[6..12]);

//             let ethertype = u16::from_be_bytes([buffer[12], buffer[13]]);

//             println!("  Source MAC:      {src}");
//             println!("  Destination MAC: {dst}");
//             println!("  EtherType:       0x{ethertype:04x}");
//         }

//         // Print the first 64 bytes.
//         print!("  Bytes: ");

//         for byte in &buffer[..n.min(64)] {
//             print!("{byte:02x} ");
//         }

//         if n > 64 {
//             print!("...");
//         }

//         println!();
//     }
// }
