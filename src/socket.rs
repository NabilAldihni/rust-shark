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

pub fn mac_address(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(":")
}
