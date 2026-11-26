use std::{
    io,
    sync::mpsc,
    thread,
};
use std::os::fd::{AsRawFd, OwnedFd};
use crate::socket::*;

#[derive(Debug)]
pub struct StreamHandler {
    #[allow(dead_code)]
    pub sender: mpsc::SyncSender<Vec<u8>>,
    pub receiver: mpsc::Receiver<Vec<u8>>,
    #[allow(dead_code)]
    handler: thread::JoinHandle<()>,
}

impl StreamHandler {
    pub fn new(max_bound: usize, interface: &str) -> Self {
        let (stream_sender, stream_receiver) = mpsc::sync_channel::<Vec<u8>>(max_bound);

        let fd: OwnedFd = open_packet_socket(interface).expect("Could not open socket");
        let mut buffer = vec![0u8; 65535];

        let handler = {
            let stream_sender = stream_sender.clone();
            thread::spawn(move || {
                loop {
                    let mut sender = libc::sockaddr_ll {
                        sll_family: 0,
                        sll_protocol: 0,
                        sll_ifindex: 0,
                        sll_hatype: 0,
                        sll_pkttype: 0,
                        sll_halen: 0,
                        sll_addr: [0; 8],
                    };

                    let mut sender_len = size_of::<libc::sockaddr_ll>() as libc::socklen_t;

                    let n = unsafe {
                        libc::recvfrom(
                            fd.as_raw_fd(),
                            buffer.as_mut_ptr().cast::<libc::c_void>(),
                            buffer.len(),
                            0,
                            (&mut sender as *mut libc::sockaddr_ll).cast(),
                            &mut sender_len,
                        )
                    };

                    if n < 0 {
                        let error = io::Error::last_os_error();

                        if error.kind() == io::ErrorKind::Interrupted {
                            continue;
                        }
                    }

                    // let n = n as usize;
                    // if n >= 14 {
                    //     let _dst = mac_address(&buffer[0..6]);
                    //     let _src = mac_address(&buffer[6..12]);
                    //     let _ethertype = u16::from_be_bytes([buffer[12], buffer[13]]);
                    // }

                    stream_sender.send(buffer.clone()).unwrap();
                }
            })
        };
        Self {
            sender: stream_sender,
            receiver: stream_receiver,
            handler,
        }
    }
}
