use ratatui::{
    Frame, layout::*, style::{Color, Style}, text::{Line, Span}, widgets::{Block, BorderType, Borders, List, ListItem, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState}
};

use crate::{
    app::{App, Screen}, 
    socket::mac_address,
    parser::*,
};

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

fn render_interface_selection_screen(app: &mut App, frame: &mut Frame) {
    if app.available_ifaces.is_empty() {
        app.fetch_interfaces();
    }

    let mut list_items = Vec::<ListItem>::new();

    for (i, iface) in app.available_ifaces.iter().enumerate() {
        // Different styling for the select iface
        if i == app.selected_iface {
            list_items.push(ListItem::new(Line::from(Span::styled(iface.name.clone(), Style::default().fg(Color::Red)))));
        } else {
            list_items.push(ListItem::new(Line::from(Span::styled(iface.name.clone(), Style::default().fg(Color::Yellow)))));
        }
    }

    let list = List::new(list_items);

    let popup_area = centered_rect(50, 50, frame.area());
    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Min(1), Constraint::Fill(1)])
        .split(popup_area);

    frame.render_widget(
        Block::default()
            .title("Select an interface")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(Color::Yellow)),
        popup_area
    );

    frame.render_widget(
        Paragraph::new("Press `j`, `k`, and `Enter` to select an interface to listen on.")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center),
        popup_chunks[0]
    );
    frame.render_widget(
        list
            .style(Style::default().fg(Color::Yellow)),
        popup_chunks[1]
    );
}

fn render_capture_screen(app: &mut App, frame: &mut Frame) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(3), Constraint::Fill(1)])
        .split(frame.area());

    frame.render_widget(Paragraph::new(format!(
        "Here is the live capture for interface: {}\n", 
        app.interface.clone().unwrap().name))
        .block(Block::default()
            .padding(Padding::horizontal(2))
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .style(Style::default().fg(Color::Yellow)),
        ),
        sections[0]
    );

    for msg in app.stream.as_ref().expect("stream not set up yet").receiver.try_iter() {
        app.packets.push(parse(&msg));
    }

    let mut list_items = Vec::<ListItem>::new();

    for packet in app.packets.iter() {
        list_items.push(ListItem::new(
            Line::from(vec![
                Span::styled(
                    "From: ",
                    Style::default().bold()
                ),
                Span::styled(
                    mac_address(&packet.link.unwrap().src_host) + " |\t",
                    Style::default().fg(Color::Gray)
                ),
                Span::styled(
                    "To: ",
                    Style::default().bold()
                ),
                Span::styled(
                    mac_address(&packet.link.unwrap().dst_host) + " |\t",
                    Style::default().fg(Color::Gray)
                ),
                Span::styled(
                    "Ethertype: ",
                    Style::default().bold()
                ),
                Span::styled(
                    format!("0x{:04x} |\t", packet.link.unwrap().ether_type),
                    Style::default().fg(Color::Gray)
                ),
                match packet.network.as_ref() {
                    Some(nw) => {
                        match nw {
                            Network::Ip(_) => {
                                Span::styled(
                                    "IP: ",
                                    Style::default().bold()
                                )
                            },
                            Network::Arp(_) => {
                                Span::styled(
                                    "ARP: ",
                                    Style::default().bold()
                                )
                            }
                        }
                    },
                    None => {
                        Span::styled(
                            "Unknown",
                            Style::default().bold()
                        )
                    }
                },
                match packet.network.as_ref() {
                    Some(nw) => {
                        match nw {
                            Network::Ip(ip) => {
                                Span::styled(
                                    format!("{:?}", ip.src),
                                    Style::default().bold()
                                )
                            },
                            Network::Arp(arp) => {
                                Span::styled(
                                    format!("{:?}", arp.sender_ip),
                                    Style::default().bold()
                                )
                            }
                        }
                    },
                    None => {
                        Span::from("")
                    }
                },
            ])
        ));
    }

    if !list_items.is_empty() {
        app.packet_list_state.select_last();
    }

    let packet_block = Block::default()
        .padding(Padding::left(2))
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .style(Style::default().fg(Color::Yellow));

    let viewport_len = packet_block.inner(sections[1]).height as usize;
    let total = app.packets.len();

    let list = List::new(list_items)
        .highlight_style(Style::default())
        .block(packet_block);

    frame.render_stateful_widget(list, sections[1], &mut app.packet_list_state);

    if total > viewport_len && viewport_len > 0 {
        let max_scroll = total.saturating_sub(viewport_len);
        let mut scrollbar_state = ScrollbarState::new(max_scroll + 1)
            .position(app.packet_list_state.offset().min(max_scroll))
            .viewport_content_length(viewport_len);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(Color::Yellow)),
            sections[1].inner(Margin { vertical: 1, horizontal: 0 }),
            &mut scrollbar_state,
        );
    }
}

pub fn render(app: &mut App, frame: &mut Frame) {
    match app.current_screen {
        Screen::ChooseInterface => render_interface_selection_screen(app, frame),
        Screen::LiveCapture => render_capture_screen(app, frame),
    }
}
