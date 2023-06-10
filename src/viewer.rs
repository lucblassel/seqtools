use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

struct App {
    yscroll: u16,
    xscroll: u16,
    ids: Vec<String>,
    seqs: Vec<String>,
    title: String,
    maxlen: u16,
    nseqs: u16,
    frame_height: u16,
    frame_width: u16,
    alphabet: Alphabet,
    dark: bool,
}

const NUCLEOTIDES: [char; 11] = ['A', 'a', 'T', 't', 'C', 'c', 'G', 'g', 'U', 'u', '-'];
enum Alphabet {
    Nucleic,
    Protein,
}

impl Alphabet {
    fn colorize(&self, c: char) -> Color {
        let c = c.to_ascii_uppercase();
        match self {
            Self::Nucleic => match c {
                'A' => Color::Red,
                'C' => Color::Yellow,
                'G' => Color::Blue,
                'T' | 'U' => Color::Green,
                _ => Color::White,
            },
            Self::Protein => match c {
                'A' | 'I' | 'L' | 'M' | 'F' | 'W' | 'V' => Color::Blue,
                'K' | 'R' => Color::Red,
                'E' | 'D' => Color::Magenta,
                'N' | 'Q' | 'S' | 'T' => Color::Green,
                'C' => Color::LightMagenta,
                'G' => Color::LightRed,
                'P' => Color::Yellow,
                'H' | 'Y' => Color::Cyan,
                _ => Color::White,
            },
        }
    }
}

impl App {
    fn new(ids: Vec<String>, seqs: Vec<String>, title: String) -> App {
        let maxlen = seqs.iter().map(|seq| seq.len() as u16).max().unwrap_or(0);
        let nseqs = seqs.len() as u16;

        let alphabet = match seqs
            .iter()
            .any(|seq| seq.chars().any(|c| !NUCLEOTIDES.contains(&c)))
        {
            true => Alphabet::Protein,
            false => Alphabet::Nucleic,
        };

        App {
            yscroll: 0,
            xscroll: 0,
            title,
            ids,
            seqs,
            maxlen,
            nseqs,
            frame_height: 0,
            frame_width: 0,
            alphabet,
            dark: true,
        }
    }

    fn set_frame(&mut self, rect: &Rect) {
        self.frame_height = rect.height;
        self.frame_width = rect.width;
    }

    fn scroll_right(&mut self) {
        if self.xscroll < self.maxlen.saturating_sub(self.frame_width) {
            self.xscroll += 1;
        }
    }

    fn scroll_left(&mut self) {
        self.xscroll = self.xscroll.saturating_sub(1);
    }

    fn scroll_down(&mut self) {
        if self.yscroll < self.nseqs.saturating_sub(self.frame_height) {
            self.yscroll += 1;
        }
    }

    fn scroll_up(&mut self) {
        self.yscroll = self.yscroll.saturating_sub(1);
    }

    fn scroll_top(&mut self) {
        self.yscroll = 0;
    }

    fn scroll_bottom(&mut self) {
        self.yscroll = self.nseqs.saturating_sub(self.frame_height);
    }

    fn scroll_start(&mut self) {
        self.xscroll = 0;
    }

    fn scroll_end(&mut self) {
        self.xscroll = self.maxlen.saturating_sub(self.frame_width);
    }

    fn toggle_dark(&mut self) {
        self.dark = !self.dark
    }
}

pub fn render_view(
    ids: Vec<String>,
    seqs: Vec<String>,
    title: String,
) -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let tick_rate = Duration::from_millis(1000);
    let app = App::new(ids, seqs, title);
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                    KeyCode::Char('T') | KeyCode::Char('t') => app.toggle_dark(),
                    KeyCode::Up => app.scroll_up(),
                    KeyCode::Down => app.scroll_down(),
                    KeyCode::Right => app.scroll_right(),
                    KeyCode::Left => app.scroll_left(),
                    KeyCode::PageUp => app.scroll_top(),
                    KeyCode::PageDown => app.scroll_bottom(),
                    KeyCode::Home => app.scroll_start(),
                    KeyCode::End => app.scroll_end(),
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();

    let block = Block::default().style(Style::default().fg(Color::Black));
    f.render_widget(block, size);

    let bg = if app.dark { Color::Black } else { Color::White };

    let fg = if app.dark { Color::White } else { Color::Black };

    let create_block = |title| {
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(bg).fg(fg))
            .title(Span::styled(
                title,
                Style::default().add_modifier(Modifier::BOLD),
            ))
    };

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(5)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(20),
            Constraint::Length(5),
        ])
        .split(size);

    let title = Paragraph::new(Span::from(app.title.clone()))
        .style(Style::default().add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(create_block("File"));
    f.render_widget(title, main_layout[0]);

    let help_layout = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(main_layout[2]);

    let navigation_help = Paragraph::new(vec![
        Spans::from("  ← → ↑ ↓    Scroll Left/Right/Up/Down"),
        Spans::from("  PgUp PdDn  Scroll to Top/Bottom"),
        Spans::from("  Home End   Scroll to Beginning/End"),
    ])
    .style(Style::default())
    .block(create_block("Navigation:"));
    f.render_widget(navigation_help, help_layout[0]);

    let display_help = Paragraph::new(vec![
        Spans::from("  T  Toggle light/dark mode"),
        Spans::from("  ?  Toggle Help"),            //TODO
        Spans::from("  R  Toggle fore/background"), //TODO
    ])
    .style(Style::default())
    .block(create_block("Rendering:"));
    f.render_widget(display_help, help_layout[1]);

    let alignment_layout = Layout::default()
        .direction(Direction::Horizontal)
        .vertical_margin(1)
        .constraints([Constraint::Length(10), Constraint::Min(20)].as_ref())
        .split(main_layout[1]);

    app.set_frame(&alignment_layout[1]);

    let seq_ids: Vec<_> = app.ids.iter().map(|id| Spans::from(id.clone())).collect();

    let seqs: Vec<_> = app
        .seqs
        .iter()
        .map(|seq| {
            let colored: Vec<_> = seq
                .chars()
                .map(|c| {
                    let color = app.alphabet.colorize(c);
                    Span::styled(c.to_string(), Style::default().bg(color))
                })
                .collect();
            Spans::from(colored)
        })
        .collect();

    // let id_par = Paragraph::new(rendered.ids)
    let id_par = Paragraph::new(seq_ids)
        .style(Style::default())
        .block(create_block("Id"))
        .scroll((app.yscroll, 0))
        .alignment(Alignment::Right);
    f.render_widget(id_par, alignment_layout[0]);
    // let seq_par = Paragraph::new(rendered.seqs)
    let seq_par = Paragraph::new(seqs)
        .style(Style::default())
        .block(create_block("Sequence"))
        .alignment(Alignment::Left)
        .scroll((app.yscroll, app.xscroll));
    f.render_widget(seq_par, alignment_layout[1]);
}
