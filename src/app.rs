use netdev::{get_interfaces, Interface};
use std::thread;

use std::io;
use std::mem::size_of;
use std::os::fd::{AsRawFd, OwnedFd};

use crate::{
    app::Screen::LiveCapture,
    socket::*
};

#[derive(Debug, Default)]
pub enum Screen {
    #[default]
    ChooseInterface,
    LiveCapture
}

#[derive(Debug, Default)]
pub struct App {
    pub current_screen: Screen,

    pub available_ifaces: Vec<Interface>,
    pub selected_iface: usize,

    pub interface: Option<Interface>,
    pub packets: Vec<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn fetch_interfaces(&mut self) {
        self.available_ifaces = get_interfaces();
    }

    pub fn select_prev_iface(&mut self) {
        if self.selected_iface == 0 {
            self.selected_iface = (self.available_ifaces.len() - 1).into();
        } else {
            self.selected_iface -= 1;
        }
    }

    pub fn select_next_iface(&mut self) {
        if self.selected_iface == self.available_ifaces.len() - 1 {
            self.selected_iface = 0;
        } else {
            self.selected_iface += 1
        };
    }

    pub fn confirm_selected_iface(&mut self) {
        self.interface = Some(self.available_ifaces[self.selected_iface].clone());
        self.current_screen = LiveCapture;

        let fd: OwnedFd = open_packet_socket(&self.interface.as_ref().unwrap().name).unwrap();
        let mut buffer = vec![0u8; 65535];

        let _handle = thread::spawn(move || {
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

                let n = n as usize;
                if n >= 14 {
                    let _dst = mac_address(&buffer[0..6]);
                    let _src = mac_address(&buffer[6..12]);
                    let _ethertype = u16::from_be_bytes([buffer[12], buffer[13]]);
                }
            }
        });
    }
}
