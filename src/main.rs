use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    },
    execute,
    terminal::LeaveAlternateScreen,
};
use futures::{SinkExt, StreamExt};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{palette::tailwind, Color, Modifier, Style},
    text::{Line, Text},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, HighlightSpacing, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, TableState, Wrap,
    },
    Frame, Terminal,
};
use serde::{Deserialize, Serialize};
use std::io;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

mod cschart;
mod term;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct HrTicker {
    pub e: String, // Event type
    pub E: u64,    // Event time
    pub s: String, // Symbol
    #[serde(deserialize_with = "deserialize_f32_from_string")]
    pub p: f32, // Price change
    #[serde(deserialize_with = "deserialize_f32_from_string")]
    pub P: f32, // Price change percent
    #[serde(deserialize_with = "deserialize_f32_from_string")]
    pub w: f32, // Weighted average price
    #[serde(deserialize_with = "deserialize_f32_from_string")]
    pub c: f32, // Last price
    #[serde(deserialize_with = "deserialize_f32_from_string")]
    pub Q: f32, // Last quantity
    #[serde(deserialize_with = "deserialize_f32_from_string")]
    pub o: f32, // Open price
    #[serde(deserialize_with = "deserialize_f32_from_string")]
    pub h: f32, // High price
    #[serde(deserialize_with = "deserialize_f32_from_string")]
    pub l: f32, // Low price
    pub v: String, // Total traded base asset volume
    pub q: String, // Total traded quote asset volume
    pub O: u64,    // Statistics open time
    pub C: u64,    // Statistics close time
    pub F: u64,    // First trade ID
    pub L: u64,    // Last trade ID
    pub n: u64,    // Total number of trades
}

#[derive(Clone, Debug)]
pub struct Tickers {
    pub tickers: Arc<Mutex<Vec<HrTicker>>>,
}

impl Tickers {
    pub fn new() -> Self {
        Self {
            tickers: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl std::fmt::Display for HrTicker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HrTicker {{ s: {} }}", self.s)
    }
}

fn deserialize_f32_from_string<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<f32>().map_err(serde::de::Error::custom)
}

async fn subscribe_to_ticker(
    tx: mpsc::Sender<Vec<HrTicker>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = "wss://fstream.binance.com/ws/!ticker@arr";
    let (ws_stream, _) = connect_async(url).await?;
    let (_, mut read) = ws_stream.split();

    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                let parsed: Vec<HrTicker> = serde_json::from_str(&text).unwrap();
                tx.send(parsed).await.unwrap();
            }
        }
    });

    Ok(())
}

const PALETTES: [tailwind::Palette; 4] = [
    tailwind::BLUE,
    tailwind::EMERALD,
    tailwind::INDIGO,
    tailwind::RED,
];

struct TableColors {
    buffer_bg: Color,
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    selected_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
    footer_border_color: Color,
}

impl TableColors {
    const fn new(color: &tailwind::Palette) -> Self {
        Self {
            buffer_bg: tailwind::SLATE.c950,
            header_bg: color.c900,
            header_fg: tailwind::SLATE.c200,
            row_fg: tailwind::SLATE.c200,
            selected_style_fg: color.c400,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            footer_border_color: color.c400,
        }
    }
}

const ITEM_HEIGHT: usize = 1;
const INFO_TEXT: &str =
    "(Esc) quit | (↑) move up | (↓) move down | (→) next color | (←) previous color";

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Mode {
    #[default]
    Running,
    Quit,
}

struct App {
    mode: Mode,
    state: TableState,
    scroll_state: ScrollbarState,
    scroll_position: usize,
    colors: TableColors,
    color_index: usize,
    ticker_length: usize,
    show_chart: bool,
}

impl App {
    fn new() -> Self {
        Self {
            mode: Mode::Running,
            state: TableState::default(),
            scroll_state: ScrollbarState::new(20),
            scroll_position: 0,
            colors: TableColors::new(&PALETTES[0]),
            color_index: 2,
            ticker_length: 25,
            show_chart: false,
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut Terminal<impl Backend>,
        tickers: Arc<Mutex<Vec<HrTicker>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        while self.is_running() {
            terminal.draw(|f| {
                let tickers_clone = Arc::clone(&tickers);
                self.ticker_length = tickers_clone.lock().unwrap().len();
                ui(f, self, tickers_clone);
            })?;
            self.scroll_state = ScrollbarState::new(self.ticker_length * ITEM_HEIGHT)
                .position(self.scroll_position);
            self.handle_events().ok();
        }
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.mode != Mode::Quit
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.ticker_length - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_position += ITEM_HEIGHT;
        self.scroll_state = self.scroll_state.position(self.scroll_position);
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.ticker_length - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_position -= ITEM_HEIGHT;
        self.scroll_state = self.scroll_state.position(self.scroll_position);
    }

    pub fn next_color(&mut self) {
        self.color_index = (self.color_index + 1) % PALETTES.len();
    }

    pub fn previous_color(&mut self) {
        let count = PALETTES.len();
        self.color_index = (self.color_index + count - 1) % count;
    }

    pub fn set_colors(&mut self) {
        self.colors = TableColors::new(&PALETTES[self.color_index]);
    }

    fn handle_events(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let timeout = Duration::from_millis(10);
        match term::next_event(timeout)? {
            Some(Event::Key(key)) if key.kind == KeyEventKind::Press => self.handle_key_press(key),
            _ => {}
        }
        Ok(())
    }

    fn handle_key_press(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.mode = Mode::Quit,
            KeyCode::Char('j') | KeyCode::Down => self.next(),
            KeyCode::Char('k') | KeyCode::Up => self.previous(),
            KeyCode::Char('l') | KeyCode::Right => self.next_color(),
            KeyCode::Char('h') | KeyCode::Left => self.previous_color(),
            KeyCode::Enter => {
                self.show_chart = true;
            }
            _ => {}
        };
    }
}

fn ui(f: &mut Frame, app: &mut App, tickers: Arc<Mutex<Vec<HrTicker>>>) {
    let rects = Layout::vertical([Constraint::Min(5), Constraint::Length(3)]).split(f.size());

    if app.show_chart {
        let area = f.size();
        f.render_widget(Clear, area);

        // tokio::spawn(async move {
        //     let chart = cschart::display_cs().await;
        // });
    } else {
        app.set_colors();

        render_table(f, app, rects[0], tickers);

        render_scrollbar(f, app, rects[0]);

        render_footer(f, app, rects[1]);
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

fn render_table(f: &mut Frame, app: &mut App, area: Rect, tickers: Arc<Mutex<Vec<HrTicker>>>) {
    let header_style = Style::default()
        .fg(app.colors.header_fg)
        .bg(app.colors.header_bg);

    let header = [
        "Symbol",
        "Last",
        "Percent Change",
        "Open",
        "High",
        "Low",
        "Volume",
    ]
    .into_iter()
    .map(Cell::from)
    .collect::<Row>()
    .style(header_style)
    .height(1);

    let selected_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(app.colors.selected_style_fg);

    let rows = tickers
        .lock()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(i, ticker)| {
            let color = if i % 2 == 0 {
                app.colors.normal_row_color
            } else {
                app.colors.alt_row_color
            };

            Row::new(vec![
                Cell::from(ticker.s.clone()),
                Cell::from(ticker.c.to_string()),
                Cell::from(ticker.P.to_string()),
                Cell::from(ticker.o.to_string()),
                Cell::from(ticker.h.to_string()),
                Cell::from(ticker.l.to_string()),
                Cell::from(ticker.v.clone()),
            ])
            .style(Style::default().fg(app.colors.row_fg).bg(color))
            .height(1)
        })
        .collect::<Vec<Row>>();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Crypto Tickers"),
    )
    .highlight_style(selected_style)
    .highlight_spacing(HighlightSpacing::default());

    f.render_stateful_widget(table, area, &mut app.state);
}

fn render_scrollbar(f: &mut Frame, app: &mut App, area: Rect) {
    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None),
        area.inner(Margin {
            vertical: 1,
            horizontal: 1,
        }),
        &mut app.scroll_state,
    );
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let info_footer = Paragraph::new(Line::from(INFO_TEXT))
        .style(
            Style::default()
                .fg(app.colors.row_fg)
                .bg(app.colors.buffer_bg),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(app.colors.footer_border_color)),
        );
    f.render_widget(info_footer, area);
}

fn update_tickers(new_tickers: Vec<HrTicker>, tickers: Arc<Mutex<Vec<HrTicker>>>) {
    let mut tickers = tickers.lock().unwrap();

    for new_ticker in new_tickers {
        match tickers.iter_mut().find(|t| t.s == new_ticker.s) {
            Some(existing_ticker) => {
                // Update existing ticker
                existing_ticker.p = new_ticker.p;
                existing_ticker.P = new_ticker.P;
                existing_ticker.w = new_ticker.w;
                existing_ticker.c = new_ticker.c;
                existing_ticker.Q = new_ticker.Q;
                existing_ticker.o = new_ticker.o;
                existing_ticker.h = new_ticker.h;
                existing_ticker.l = new_ticker.l;
                existing_ticker.v.clone_from(&new_ticker.v);
                existing_ticker.q.clone_from(&new_ticker.q);
                existing_ticker.O = new_ticker.O;
                existing_ticker.C = new_ticker.C;
                existing_ticker.F = new_ticker.F;
                existing_ticker.L = new_ticker.L;
                existing_ticker.n = new_ticker.n;
            }
            None => {
                // Add new ticker
                tickers.push(new_ticker);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tickers = Tickers::new();
    let mut app = App::new();
    let (tx, mut rx) = mpsc::channel::<Vec<HrTicker>>(100);
    //let app_clone = Arc::clone(&app);
    let tickers_clone = tickers.tickers.clone();
    tokio::spawn(async move {
        while let Some(results) = rx.recv().await {
            //let app = Arc::clone(&app_clone);
            update_tickers(results, tickers_clone.clone());
        }
    });

    tokio::spawn(async move {
        subscribe_to_ticker(tx).await.unwrap();
    });

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    app.run(&mut terminal, tickers.tickers)?;

    terminal.clear()?;
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
