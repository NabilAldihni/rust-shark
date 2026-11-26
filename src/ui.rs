use ratatui::{
    Frame, layout::*, style::{Color, Style}, text::{Line, Span}, widgets::{Block, BorderType, Borders, List, ListItem, Padding, Paragraph}
};

use crate::{
    app::{App, Screen}, 
    socket::mac_address,
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
        app.packets.push(msg);
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
                    mac_address(&packet[0..6]) + " |\t",
                    Style::default().fg(Color::Gray)
                ),
                Span::styled(
                    "To: ",
                    Style::default().bold()
                ),
                Span::styled(
                    mac_address(&packet[6..12]) + " |\t", 
                    Style::default().fg(Color::Gray)
                ),
                Span::styled(
                    "Ethertype: ",
                    Style::default().bold()
                ),
                Span::styled(
                    format!("0x{:04x} |\t", u16::from_be_bytes([packet[12], packet[13]])),
                    Style::default().fg(Color::Gray)
                ),
                Span::styled(
                    "Data: ",
                    Style::default().bold()
                ),
                Span::styled(
                    // format!("{:02x}", packet[0]),
                    packet[14..30]
                        .iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<String>>()
                        .join(" "),
                    Style::default().fg(Color::Gray)
                ),

            ])
        ));
    }

    let list = List::new(list_items);

    frame.render_widget(
        list
        .block(Block::default()
            .padding(Padding::left(2))
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .style(Style::default().fg(Color::Yellow)),
        ),
        sections[1]
    );
}

pub fn render(app: &mut App, frame: &mut Frame) {
    match app.current_screen {
        Screen::ChooseInterface => render_interface_selection_screen(app, frame),
        Screen::LiveCapture => render_capture_screen(app, frame),
    }
}
