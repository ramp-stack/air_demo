use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    prelude::*,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    messages: Vec<(String, Color)>,
    input: String,
    scroll: u16,
}

impl Tui {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut tui = Tui{
            terminal: Terminal::new(backend)?,
            messages: Vec::new(),
            input: String::new(),
            scroll: 0,
        };
        tui.draw()?;
        Ok(tui)
    }

    pub fn add_message(&mut self, author: String, message: String) {
        let color = Color::Indexed(author.chars().nth(7).unwrap() as u8);
        self.messages.push((format!("{author}: \"{message}\""), color));
    }

    pub fn tick(&mut self) -> io::Result<Option<String>> {
        let mut output = None;
        
        if let Some(Event::Key(key)) = event::poll(Duration::from_secs(0))?.then(|| event::read().unwrap()) && key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Char('q') if key.modifiers == event::KeyModifiers::CONTROL => {panic!("App shutting down")},
                KeyCode::Esc => {panic!("App shutting down")},

                KeyCode::Enter => {
                    output = Some(self.input.trim().to_string()).filter(|s| !s.is_empty());
                    self.input.clear();
                }

                KeyCode::Char(c) => self.input.push(c),
                KeyCode::Backspace => {self.input.pop();}

                KeyCode::Up => self.scroll = self.scroll.saturating_sub(1),
                KeyCode::Down => self.scroll = self.scroll.saturating_add(1),
                KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(10),
                KeyCode::PageDown => self.scroll = self.scroll.saturating_add(10),

                _ => {}
            }
        }


        self.draw()?;
        
        Ok(output)
    }

    fn draw(&mut self) -> io::Result<()> {
        self.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(85), Constraint::Min(3)])
                .split(f.area());

            // Build colored text for the chat
            let mut text = Text::default();
            for (msg, color) in &self.messages {
                text.lines.push(Line::from(Span::styled(
                    msg.clone(),
                    Style::default().fg(*color),
                )));
            }

            let messages = Paragraph::new(text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Chat ")
                        .title_style(Style::default().fg(Color::Cyan)),
                )
                .wrap(Wrap { trim: true })
                .scroll((self.scroll, 0));

            f.render_widget(messages, chunks[0]);

            let input = Paragraph::new(self.input.as_str())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Type your message (Enter to send, Ctrl+C to quit) "),
                )
                .style(Style::default().fg(Color::Yellow));

            f.render_widget(input, chunks[1]);

            let input_area = chunks[1];
            f.set_cursor_position((
                input_area.x + self.input.len() as u16 + 1,
                input_area.y + 1,
            ));
        })?;
        Ok(())
    }
}
impl Drop for Tui {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        execute!(io::stdout(), LeaveAlternateScreen).unwrap();
    }
}
