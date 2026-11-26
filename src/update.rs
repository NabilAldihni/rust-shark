use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, Screen};

pub fn update(app: &mut App, key_event: KeyEvent) {
    match app.current_screen {
        Screen::ChooseInterface => match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => app.quit(),
            KeyCode::Char('c') | KeyCode::Char('C') if key_event.modifiers == KeyModifiers::CONTROL => app.quit(),
            KeyCode::Up | KeyCode::Char('k') => app.select_prev_iface(),
            KeyCode::Down | KeyCode::Char('j') => app.select_next_iface(),
            KeyCode::Enter => app.confirm_selected_iface(),
            _ => {}
        },
        Screen::LiveCapture => match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => app.quit(),
            KeyCode::Char('c') | KeyCode::Char('C') if key_event.modifiers == KeyModifiers::CONTROL => app.quit(),
            _ => {}
        }
    };
}
