use netdev::{get_interfaces, Interface};
use ratatui::widgets::ListState;

use crate::{
    app::Screen::LiveCapture, parser::PacketLayers, stream::StreamHandler
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
    pub packets: Vec<PacketLayers>,
    pub packet_list_state: ListState,
    pub should_quit: bool,

    pub stream: Option<StreamHandler>,
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
        self.stream = Some(StreamHandler::new(100, &self.interface.as_ref().unwrap().name));
    }
}
