use crate::{log_reader::LogReader, settings::Settings};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};

pub struct App {
    should_exit: bool,
    settings: Settings,
    log_reader: Option<LogReader>,

    // NEW:
    table_state: TableState,
}

impl App {
    pub fn new() -> Self {
        let settings = Settings::new();

        let mut table_state = TableState::default();
        table_state.select(Some(0)); // start selected at row 0

        let mut app = App {
            should_exit: false,
            settings,
            log_reader: None,
            table_state,
        };
        app.init();
        app
    }

    fn init(&mut self) {
        self.log_reader = Some(LogReader::new(&self.settings));
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_key_events();
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        // Hard-coded table data for now
        let rows = [
            ("firefox", "12m 31s", "100%"),
            ("alacritty", "03m 10s", "50%"),
            ("code", "47m 02s", "100%"),
            ("slack", "08m 44s", "100%"),
        ];

        let header = Row::new([
            Cell::from("Class"),
            Cell::from("Duration"),
            Cell::from("Percentage"),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD));

        let table_rows = rows.iter().map(|(start, app, dur)| {
            Row::new([Cell::from(*start), Cell::from(*app), Cell::from(*dur)])
        });

        let block = Block::default().title("Hyprlog").borders(Borders::ALL);

        let table = Table::new(
            table_rows,
            [
                Constraint::Length(19), // "YYYY-MM-DD HH:MM:SS"
                Constraint::Min(10),    // app name expands
                Constraint::Length(10), // duration
            ],
        )
        .header(header)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

        // Optional: give the table some padding via layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Min(0)])
            .split(frame.area());

        frame.render_stateful_widget(table, chunks[0], &mut self.table_state);
    }

    fn handle_key_events(&mut self) {
        if let Ok(event) = crossterm::event::read() {
            if let crossterm::event::Event::Key(key_event) = event {
                match key_event.code {
                    crossterm::event::KeyCode::Up => {
                        self.select_prev_row(4); // hard-coded row count for now
                    }
                    crossterm::event::KeyCode::Down => {
                        self.select_next_row(4); // hard-coded row count for now
                    }
                    crossterm::event::KeyCode::Char('q' | 'Q') => self.should_exit = true,
                    _ => {}
                }
            }
        }
    }

    fn select_next_row(&mut self, row_count: usize) {
        if row_count == 0 {
            return;
        }
        let i = self.table_state.selected().unwrap_or(0);
        let next = if i + 1 >= row_count { 0 } else { i + 1 };
        self.table_state.select(Some(next));
    }

    fn select_prev_row(&mut self, row_count: usize) {
        if row_count == 0 {
            return;
        }
        let i = self.table_state.selected().unwrap_or(0);
        let prev = if i == 0 { row_count - 1 } else { i - 1 };
        self.table_state.select(Some(prev));
    }
}
