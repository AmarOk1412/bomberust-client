use crate::util::{ Config, Event, Events };
use std::io;
use std::thread;
use std::time::{ self, Duration, SystemTime };
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::Rect;
use tui::style::{ Style, Color, Modifier };
use tui::Terminal;
use tui::widgets::{ Block, Borders, SelectableList, Widget };
use tui::widgets::canvas::Canvas;
use tui::terminal::Frame;


#[derive(PartialEq)]
pub enum Location {
    Splash,
    ConfigureServer,
    Lobby,
    Room,
    Game
}

pub struct TuiClient<'a> {
    location: Location,
    selected_server: Option<usize>,
    servers_list: Vec<&'a str>,
}

impl<'a> TuiClient<'a> {
    pub fn new() -> TuiClient<'a> {
        TuiClient {
            location: Location::Splash,
            selected_server: Some(0),
            servers_list: vec!["Add a new server to the list", "127.0.0.1:1412"]
        }
    }

    pub fn render(&mut self) -> Result<(), failure::Error> {
        // Terminal initialization
        let stdout = io::stdout().into_raw_mode()?;
        let stdout = MouseTerminal::from(stdout);
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;

        let config = Config {
            tick_rate: Duration::from_millis(100),
            ..Default::default()
        };
        let events = Events::with_config(config);

        let ten_millis = time::Duration::from_millis(10);
        let now = time::Instant::now();

        loop {
            terminal.draw(|mut f| {
                match self.location {
                    Location::Splash => {
                        self.render_splash(&mut f);
                        self.draw_server_list(&mut f);
                    },
                    Location::ConfigureServer => {
                        self.render_splash(&mut f);
                        self.configure_new_server(&mut f);
                    }
                    _ => { println!("TODO"); }
                }
            });

            match events.next()? {
                Event::Input(input) => match input {
                    Key::Esc => {
                        break;
                    },
                    Key::Down => {
                        self.selected_server = if let Some(selected) = self.selected_server {
                            if selected >= self.servers_list.len() - 1 {
                                Some(0)
                            } else {
                                Some(selected + 1)
                            }
                        } else {
                            Some(0)
                        };
                    },
                    Key::Up => {
                        self.selected_server = if let Some(selected) = self.selected_server {
                            if selected > 0 {
                                Some(selected - 1)
                            } else {
                                Some(self.servers_list.len() - 1)
                            }
                        } else {
                            Some(0)
                        };
                    },
                    Key::Char(' ') => {
                        if self.location == Location::Splash {
                            let selected = self.selected_server.unwrap_or(0);
                            if selected == 0 {
                                self.location = Location::ConfigureServer;
                            }
                        } else {
                            self.location = Location::Splash;
                        }
                    },
                    _ => {}
                },
                Event::Tick => {
                }
            }
            thread::sleep(ten_millis);
        }

        Ok(())
    }

    fn render_splash<B: tui::backend::Backend>(&mut self, mut f: &mut Frame<B>) {
        let size = f.size();
        
        let logo_odd = vec!["                                                                    ",
"                                                                                                ",
"                >\"<                                                                            ",
"                 █╗                                                                             ",
"                █╔╝                                                                             ",
"   ██████╗  ██████╗ ███╗   ███╗██████╗ ███████╗██████╗ ██╗   ██╗███████╗████████╗               ",
"   ██╔══██╗██╔═══██╗████╗ ████║██╔══██╗██╔════╝██╔══██╗██║   ██║██╔════╝╚══██╔══╝               ",
"   ██████╔╝██║   ██║██╔████╔██║██████╔╝█████╗  ██████╔╝██║   ██║███████╗   ██║                  ",
"   ██╔══██╗██║   ██║██║╚██╔╝██║██╔══██╗██╔══╝  ██╔══██╗██║   ██║╚════██║   ██║                  ",
"   ██████╔╝╚██████╔╝██║ ╚═╝ ██║██████╔╝███████╗██║  ██║╚██████╔╝███████║   ██║       ███████╗   ",
"   ╚═════╝  ╚═════╝ ╚═╝     ╚═╝╚═════╝ ╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝       ╚══════╝   ",
"                                                        ___   ___     ___                       ",
"                                                    _ _|   | |_  |   |_  |                      ",
"                                                   | | | | |_ _| |_ _ _| |_                     ",
"                                                    \\_/|___|_|_____|_|_____|                   "];
        
        let logo = vec!["                                                            ",
"                                                                                    ",
"                -\"-                                                                ",
"                 █╗                                                                 ",
"                █╔╝                                                                 ",
"   ██████╗  ██████╗ ███╗   ███╗██████╗ ███████╗██████╗ ██╗   ██╗███████╗████████╗   ",
"   ██╔══██╗██╔═══██╗████╗ ████║██╔══██╗██╔════╝██╔══██╗██║   ██║██╔════╝╚══██╔══╝   ",
"   ██████╔╝██║   ██║██╔████╔██║██████╔╝█████╗  ██████╔╝██║   ██║███████╗   ██║      ",
"   ██╔══██╗██║   ██║██║╚██╔╝██║██╔══██╗██╔══╝  ██╔══██╗██║   ██║╚════██║   ██║      ",
"   ██████╔╝╚██████╔╝██║ ╚═╝ ██║██████╔╝███████╗██║  ██║╚██████╔╝███████║   ██║      ",
"   ╚═════╝  ╚═════╝ ╚═╝     ╚═╝╚═════╝ ╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝      ",
"                                                        ___   ___     ___           ",
"                                                    _ _|   | |_  |   |_  |          ",
"                                                   | | | | |_ _| |_ _ _| |_         ",
"                                                    \\_/|___|_|_____|_|_____|       "];
        Canvas::default()
            .block(Block::default().borders(Borders::NONE))
            .paint(|ctx| {
                let mut idx = 3;
                let line_len = logo_odd[logo_odd.len() - 1].len();
                let offset_x = (size.width - line_len as u16) / 2;
                let now = SystemTime::now();
                let odd_sec = now.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() % 2 == 0;
                if odd_sec {
                    for line in &logo_odd {
                        ctx.print(offset_x as f64, ((size.height / 2) - idx) as f64, line, Color::Yellow);
                        idx += 1;
                    }
                } else {
                    for line in &logo {
                        ctx.print(offset_x as f64, ((size.height / 2) - idx) as f64, line, Color::Yellow);
                        idx += 1;
                    }
                }
            })
            .x_bounds([0.0, size.width as f64])
            .y_bounds([0.0, ((size.height / 2) - 2) as f64])
            .render(&mut f, Rect::new(0, 0, size.width, size.height / 2));            
    }

    fn draw_server_list<B: tui::backend::Backend>(&mut self, mut f: &mut Frame<B>) {
        let size = f.size();

        let style = Style::default().fg(Color::Black).bg(Color::White);
        SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("Servers"))
                .items(&self.servers_list)
                .select(self.selected_server)
                .highlight_style(Style::default().fg(Color::LightGreen).modifier(Modifier::BOLD))
                .highlight_symbol(">")
                .render(&mut f, Rect::new(0, size.height / 2, size.width, size.height / 2));       
    }

    fn configure_new_server<B: tui::backend::Backend>(&mut self, mut f: &mut Frame<B>) {
        let size = f.size();
        Canvas::default()
            .block(Block::default().borders(Borders::ALL).title("Configure").style(Style::default().bg(Color::White)))
            .paint(|ctx| {
            })
            .x_bounds([0.0, size.width as f64])
            .y_bounds([0.0, ((size.height / 2) - 2) as f64])
            .render(&mut f, Rect::new(0, size.height / 2, size.width, size.height / 2));            
    }
}