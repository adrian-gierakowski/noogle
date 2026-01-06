use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Widget, Wrap},
    Frame, Terminal,
};
use std::{error::Error, io};
use unicode_width::UnicodeWidthStr;
use crate::model::DocItem;
use crate::search::search;
use crate::details::render_doc;

struct App<'a> {
    input: String,
    data: &'a [DocItem],
    results: Vec<crate::search::SearchResult<'a>>,
    state: ListState,
}

impl<'a> App<'a> {
    fn new(data: &'a [DocItem]) -> App<'a> {
        App {
            input: String::new(),
            data,
            results: Vec::new(),
            state: ListState::default(),
        }
    }

    fn update_search(&mut self) {
        if self.input.is_empty() {
             self.results = self.data.iter().map(|d| crate::search::SearchResult { doc: d, score: 0 }).take(100).collect();
        } else {
             self.results = search(&self.input, self.data);
        }
        self.state.select(if self.results.is_empty() { None } else { Some(0) });
    }

    fn on_key(&mut self, c: char) {
        self.input.push(c);
        self.update_search();
    }

    fn on_backspace(&mut self) {
        self.input.pop();
        self.update_search();
    }

    fn select_next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.results.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn select_previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.results.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

pub fn run_tui(data: &[DocItem]) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(data);
    app.update_search(); // Initial populate

    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc => return Ok(()),
                KeyCode::Char(c) => app.on_key(c),
                KeyCode::Backspace => app.on_backspace(),
                KeyCode::Down => app.select_next(),
                KeyCode::Up => app.select_previous(),
                KeyCode::Enter => {
                    // Open in browser or expand?
                    if let Some(selected) = app.state.selected() {
                        if let Some(_result) = app.results.get(selected) {
                             // Attempt to open documentation URL if possible?
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

struct HyperlinkWidget<'a> {
    url: &'a str,
    label: &'a str,
}

impl<'a> Widget for HyperlinkWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (x, y) = (area.x, area.y);
        // Render label text normally so Ratatui sets up the cells correctly
        buf.set_string(x, y, self.label, Style::default());

        // Now inject the escape sequences into the symbols of the first and last cells.
        // This preserves layout (ratatui thinks it's just text) but terminal renders link.

        let start_seq = format!("\x1b]8;;{}\x1b\\", self.url);
        let end_seq = "\x1b]8;;\x1b\\";

        // Modify first cell
        if area.width > 0 {
            let first_cell = buf.get_mut(x, y);
            let symbol = first_cell.symbol().to_string();
            first_cell.set_symbol(&format!("{}{}", start_seq, symbol));
        }

        // Modify last cell of the label
        // We need to calculate where the label ends.
        let width = self.label.width();
        if width > 0 {
            let end_x = x + (width as u16) - 1;

            // Even if truncated, we must close the sequence on the last visible cell
            let effective_end_x = std::cmp::min(end_x, area.right().saturating_sub(1));

            if effective_end_x >= x {
                 let last_cell = buf.get_mut(effective_end_x, y);
                 let symbol = last_cell.symbol().to_string();
                 last_cell.set_symbol(&format!("{}{}", symbol, end_seq));
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(f.size());

    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Search"));
    f.render_widget(input, chunks[0]);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(chunks[1]);

    let items: Vec<ListItem> = app
        .results
        .iter()
        .map(|res| {
            let item = res.doc;
            ListItem::new(Line::from(vec![
                Span::styled(item.title(), Style::default().add_modifier(Modifier::BOLD)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Results"))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    f.render_stateful_widget(list, main_chunks[0], &mut app.state);

    // Detail view
    if let Some(selected_index) = app.state.selected() {
        if let Some(res) = app.results.get(selected_index) {
            let item = res.doc;
            let block = Block::default().borders(Borders::ALL).title("Documentation");
            let inner_area = block.inner(main_chunks[1]);
            f.render_widget(block, main_chunks[1]);

            let (text, url) = render_doc(item);

            if let Some(u) = url {
                // Split inner area: Text takes available - 1 line, Link takes bottom 1 line
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
                    .split(inner_area);

                let p = Paragraph::new(text)
                    .wrap(Wrap { trim: false })
                    .scroll((0, 0));
                f.render_widget(p, chunks[0]);

                let link_widget = HyperlinkWidget {
                    url: &u,
                    label: "Jump to Source Code",
                };
                f.render_widget(link_widget, chunks[1]);

            } else {
                let p = Paragraph::new(text)
                    .wrap(Wrap { trim: false })
                    .scroll((0, 0));
                f.render_widget(p, inner_area);
            }
        }
    } else {
         let block = Block::default().borders(Borders::ALL).title("Documentation");
         f.render_widget(block, main_chunks[1]);
    }
}
