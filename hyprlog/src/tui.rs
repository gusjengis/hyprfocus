use ratatui::style::Color;

use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};

use crate::log_reader::LogReader;
use crate::settings::Settings;

// --- PALETTE (neon / high-contrast) ---
const BG: Color = Color::Rgb(10, 10, 16);

const PANEL_BG: Color = Color::Rgb(18, 18, 28);
const BORDER: Color = Color::Rgb(0, 255, 200);

const HEADER_FG: Color = Color::Rgb(0, 0, 0);
const HEADER_BG: Color = Color::Rgb(255, 210, 0);

const ROW_FG: Color = Color::Rgb(235, 235, 245);
const ROW_BG_A: Color = Color::Rgb(24, 24, 40);
const ROW_BG_B: Color = Color::Rgb(18, 18, 32);

const HILITE_FG: Color = Color::Rgb(0, 0, 0);
const HILITE_BG: Color = Color::Rgb(255, 0, 140);

const SYMBOL_FG: Color = Color::Rgb(0, 255, 200); // for the ">> "use crate::{log_reader::LogReader, settings::Settings};

// --- COLUMN COLORS ---
const COL_CLASS_FG: Color = Color::Rgb(120, 200, 255);
const COL_CLASS_BG_A: Color = Color::Rgb(20, 30, 48);
const COL_CLASS_BG_B: Color = Color::Rgb(16, 24, 40);

const COL_DURATION_FG: Color = Color::Rgb(180, 255, 180);
const COL_DURATION_BG_A: Color = Color::Rgb(20, 42, 28);
const COL_DURATION_BG_B: Color = Color::Rgb(16, 34, 24);

const COL_PERCENT_FG: Color = Color::Rgb(255, 180, 180);
const COL_PERCENT_BG_A: Color = Color::Rgb(48, 20, 28);
const COL_PERCENT_BG_B: Color = Color::Rgb(40, 16, 24);

pub struct App {
    should_exit: bool,
    settings: Settings,
    log_reader: Option<LogReader>,
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
        let rows = [
            ("firefox", "12m 31s", "100%"),
            ("alacritty", "03m 10s", "50%"),
            ("code", "47m 02s", "100%"),
            ("slack", "08m 44s", "100%"),
        ];

        // Outer background (so the whole screen isn't default gray/black)
        frame.render_widget(
            Block::default().style(Style::default().bg(BG)),
            frame.area(),
        );

        let header = Row::new([
            Cell::from("Class").style(
                Style::default()
                    .fg(Color::Black)
                    .bg(COL_CLASS_FG)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Duration").style(
                Style::default()
                    .fg(Color::Black)
                    .bg(COL_DURATION_FG)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Percentage").style(
                Style::default()
                    .fg(Color::Black)
                    .bg(COL_PERCENT_FG)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        let table_rows = rows.iter().enumerate().map(|(i, (class, dur, pct))| {
            let (class_bg, dur_bg, pct_bg) = if i % 2 == 0 {
                (COL_CLASS_BG_A, COL_DURATION_BG_A, COL_PERCENT_BG_A)
            } else {
                (COL_CLASS_BG_B, COL_DURATION_BG_B, COL_PERCENT_BG_B)
            };

            Row::new([
                Cell::from(*class).style(Style::default().fg(COL_CLASS_FG).bg(class_bg)),
                Cell::from(*dur).style(Style::default().fg(COL_DURATION_FG).bg(dur_bg)),
                Cell::from(*pct).style(Style::default().fg(COL_PERCENT_FG).bg(pct_bg)),
            ])
        });
        let block = Block::default()
            .title("Hyprlog")
            .borders(Borders::ALL)
            .style(Style::default().bg(PANEL_BG))
            .border_style(Style::default().fg(BORDER).bg(PANEL_BG));

        let table = Table::new(
            table_rows,
            [
                Constraint::Min(16),    // Class (grow)
                Constraint::Length(10), // Duration
                Constraint::Length(10), // Percentage
            ],
        )
        .header(header)
        .block(block)
        // selected row highlight:
        .highlight_style(
            Style::default()
                .fg(HILITE_FG)
                .bg(HILITE_BG)
                .add_modifier(Modifier::BOLD),
        )
        // little "cursor" marker (color it by styling the whole highlighted row + symbol fg via reversed bg)
        .highlight_symbol(">> ");

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Min(0)])
            .split(frame.area());

        frame.render_stateful_widget(table, chunks[0], &mut self.table_state);

        // Optional: color the highlight symbol by drawing it over the left margin area.
        // (If you want this, tell me your ratatui version; there are a couple clean ways.)
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
