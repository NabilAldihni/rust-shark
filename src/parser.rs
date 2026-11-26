use std::net::Ipv4Addr;

// Defines the ethernet layer
#[derive(Debug, Clone, Copy)]
pub struct EthernetLayer {
    pub dst_host: [u8; 6],
    pub src_host: [u8; 6],
    pub ether_type: u16
}

// Defines the ARP layer
#[derive(Debug, Clone, Copy)]
pub struct ArpLayer {
    pub hrd_format: u16,
    pub pro_format: u16,
    pub hrd_addr_len: u8,
    pub pro_addr_len: u8,
    pub op_code: u16,
    pub sender_mac: [u8; 6],
    pub sender_ip: Ipv4Addr,
    pub target_mac: [u8; 6],
    pub target_ip: Ipv4Addr
}

// Defines the IP(v4) layer
#[derive(Debug, Clone, Copy)]
pub struct IpLayer {
    pub hdr_len: u8,
    pub version: u8,
    pub tos: u8,
    pub len: u16,
    pub id: u16,
    pub offset: u16,
    pub ttl: u8,
    pub protocol: u8,
    pub checksum: u16,
    pub src: Ipv4Addr,
    pub dst: Ipv4Addr
}

// Defines the ICMP layer
#[derive(Debug, Clone, Copy)]
pub struct IcmpLayer {
    pub icmp_type: u8,
    pub icmp_code: u8,
    pub icmp_sum: u16
}

// Defines the TCP layer
#[derive(Debug, Clone)]
pub struct TcpLayer {
    pub src_port: u16,
    pub dst_port: u16,
    pub seq_num: u32,
    pub ack: u32,
    pub offset: u8, // header length in num words (ie 32 bits)
    pub flags: u8,
    pub window: u16,
    pub checksum: u16,
    pub urg_ptr: u16,
    pub options: Vec<u8>
}

// Defines the UDP layer
#[derive(Debug, Clone, Copy)]
pub struct UdpLayer {
    pub src_port: u16,
    pub dst_port: u16,
    pub len: u16,
    pub checksum: u16
}

// Defines the layers of a captured packet
#[derive(Debug, Default)]
pub struct PacketLayers {
    pub link: Option<EthernetLayer>, // link layer
    pub network: Option<Network>, // network layer
    pub transport: Option<Transport>, // transport layer
    pub data_offset: usize // Note: raw_packet[data_offset:] is the data body
}

// enum of currently supported network layer protocols
#[derive(Debug)]
pub enum Network { Ip(IpLayer), Arp(ArpLayer) }

// enum of currently supported transport layer protocols
#[derive(Debug)]
pub enum Transport { Tcp(TcpLayer), Udp(UdpLayer), Icmp(IcmpLayer) }

impl EthernetLayer {

    // Parse the ethernet header and determine what is the relative
    // offset of the data body
    pub fn parse(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < 14 {
            return None;
        }
        let dst_host: [u8; 6] = bytes[0..6].try_into().ok()?;
        let src_host: [u8; 6] = bytes[6..12].try_into().ok()?;
        let eth_layer = EthernetLayer {
            dst_host,
            src_host,
            ether_type: u16::from_be_bytes([bytes[12], bytes[13]])
        };
        return Some((eth_layer, 14));
    }
}

impl ArpLayer {

    // Parse the arp header and determine what is the relative
    // offset of the data body (even if this is useless since
    // arp never has a payload, just to be consistent)
    pub fn parse(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < 28 {
            return None;
        }
        let sender_mac: [u8; 6] = bytes[8..14].try_into().ok()?;
        let target_mac: [u8; 6] = bytes[18..24].try_into().ok()?;
        let arp_layer = ArpLayer {
            hrd_format: u16::from_be_bytes([bytes[0], bytes[1]]),
            pro_format: u16::from_be_bytes([bytes[2], bytes[3]]),
            hrd_addr_len: bytes[4],
            pro_addr_len: bytes[5],
            op_code: u16::from_be_bytes([bytes[6], bytes[7]]),
            sender_mac,
            sender_ip: Ipv4Addr::new(bytes[14], bytes[15], bytes[16], bytes[17]),
            target_mac,
            target_ip: Ipv4Addr::new(bytes[24], bytes[25], bytes[26], bytes[27])
        };
        return Some((arp_layer, 28));
    }
}

impl IpLayer {

    // Parse the ip header and determine what is the relative
    // offset of the data body
    pub fn parse(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < 20 {
            return None;
        }
        let hdr_len = bytes[0] >> 4;
        let version = bytes[0] & 0x0F;
        let ip_layer = IpLayer {
            hdr_len,
            version,
            tos: bytes[1],
            len: u16::from_be_bytes([bytes[2], bytes[3]]),
            id: u16::from_be_bytes([bytes[4], bytes[5]]),
            offset: u16::from_be_bytes([bytes[6], bytes[7]]),
            ttl: bytes[8],
            protocol: bytes[9],
            checksum: u16::from_be_bytes([bytes[10], bytes[11]]),
            src: Ipv4Addr::new(bytes[12], bytes[13], bytes[14], bytes[15]),
            dst: Ipv4Addr::new(bytes[16], bytes[17], bytes[18], bytes[19])
        };
        return Some((ip_layer, 20));
    }
}

impl IcmpLayer {
    // Parse the icmp header and determine what is the relative
    // offset of the data body
    pub fn parse(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < 4 {
            return None;
        }
        let icmp_layer = IcmpLayer {
            icmp_type: bytes[0],
            icmp_code: bytes[1],
            icmp_sum: u16::from_be_bytes([bytes[2], bytes[3]])
        };
        return Some((icmp_layer, 4));
    }
}

impl TcpLayer {
    // Parse the tcp header and determine what is the relative
    // offset of the data body (sometimes there's no body but
    // just to be consisten we'll still return offset)
    pub fn parse(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < 20 {
            return None;
        }
        let offset = bytes[12] >> 4;
        let hdr_len = offset as usize * 4;
        if offset < 5 || bytes.len() < hdr_len {
            return None;
        }
        let tcp_layer = TcpLayer {
            src_port: u16::from_be_bytes([bytes[0], bytes[1]]),
            dst_port: u16::from_be_bytes([bytes[2], bytes[3]]),
            seq_num: u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            ack: u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
            offset,
            flags: bytes[13],
            window: u16::from_be_bytes([bytes[14], bytes[15]]),
            checksum: u16::from_be_bytes([bytes[16], bytes[17]]),
            urg_ptr: u16::from_be_bytes([bytes[18], bytes[19]]),
            options: bytes[20..hdr_len].to_vec(),
        };
        return Some((tcp_layer, hdr_len));
    }
}

impl UdpLayer {
    pub fn parse(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < 8 {
            return None;
        }
        let udp_layer = UdpLayer {
            src_port: u16::from_be_bytes([bytes[0], bytes[1]]),
            dst_port: u16::from_be_bytes([bytes[2], bytes[3]]),
            len: u16::from_be_bytes([bytes[4], bytes[5]]),
            checksum: u16::from_be_bytes([bytes[6], bytes[7]])
        };
        return Some((udp_layer, 8));
    } 
}

pub fn parse(bytes: &[u8]) -> PacketLayers {
    let mut packet = PacketLayers { ..Default::default() };
    let _ = parse_helper(&mut packet, bytes);
    return packet;
}

pub fn parse_helper(packet: &mut PacketLayers, bytes: &[u8]) -> Option<()> {
    let mut offset = 0 as usize;
    let (eth_lyr, sum_offset) = EthernetLayer::parse(bytes)?;
    packet.link = Some(eth_lyr);
    offset += sum_offset;
    packet.data_offset = offset;

    let (network, sum_offset, protocol) = match eth_lyr.ether_type {
        0x0800 => IpLayer::parse(&bytes[offset..]).map(|(l, o)| (Network::Ip(l), o, l.protocol)),
        0x0806 => ArpLayer::parse(&bytes[offset..]).map(|(l, o)| (Network::Arp(l), o, 0)),
        _ => return None,
    }?;
    packet.network = Some(network);
    offset += sum_offset;
    packet.data_offset = offset;

    let (transport, sum_offset) = match protocol {
        1 => IcmpLayer::parse(&bytes[offset..]).map(|(l, o)| (Transport::Icmp(l), o)),
        6 => TcpLayer::parse(&bytes[offset..]).map(|(l, o)| (Transport::Tcp(l), o)),
        17 => UdpLayer::parse(&bytes[offset..]).map(|(l, o)| (Transport::Udp(l), o)),
        _ => return None,
    }?;
    packet.transport = Some(transport);
    offset += sum_offset;
    packet.data_offset = offset;

    return Some(());
}

// Just an example dummy function on how to use PacketLayers
pub fn dummy_print_packet(packet: PacketLayers) {
    match &packet.link {
        Some(eth) => println!("{:?} -> {:?}", eth.src_host, eth.dst_host),
        _ => print!("No link layer!")
    }
    match &packet.network {
        Some(Network::Ip(ip)) => println!("{} -> {}", ip.src, ip.dst),
        Some(Network::Arp(arp)) => println!("ARP op {}", arp.op_code),
        _ => print!("No network layer!")
    }
    match &packet.transport {
        Some(Transport::Icmp(icmp)) => println!("Code {}, Type {}", icmp.icmp_code, icmp.icmp_type),
        Some(Transport::Tcp(tcp)) => println!("{} -> {}", tcp.src_port, tcp.dst_port),
        Some(Transport::Udp(udp)) => println!("{} -> {}", udp.src_port, udp.dst_port),
        _ => print!("No transport layer!")
    }
}
