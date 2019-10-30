use crate::util::{ Config, Event, Events };
use std::fs::{ self, File };
use std::io::{ stdout, Write };
use std::path::Path;
use std::thread;
use std::time::{ self, Duration, SystemTime };
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::style::{ Style, Color, Modifier };
use tui::Terminal;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::widgets::{ Block, Borders, Paragraph, SelectableList, Text, Widget };
use tui::widgets::canvas::Canvas;
use tui::terminal::Frame;

// TODO Layout

#[derive(PartialEq)]
pub enum Location {
    Splash,
    ConfigureServer,
    Lobby,
    Room,
    Game
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerInfo {
    pub name: String,
    pub hostname: String,
    pub certificate: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientConfig {
    servers: Vec<ServerInfo>,
    default_playername: String,
}

pub struct TuiClient {
    location: Location,
    selected_item: Option<usize>,
    items_len: usize,
    new_server_info: Option<ServerInfo>,
    config: ClientConfig,
}

impl TuiClient {
    pub fn new() -> TuiClient {
        let mut config = ClientConfig {
            servers: Vec::new(),
            default_playername: String::new()
        };
        if Path::new("config.json").is_file() {
            let content = fs::read_to_string("config.json").unwrap_or(String::new());
            config = serde_json::from_str(&content).unwrap_or(ClientConfig {
                servers: Vec::new(),
                default_playername: String::new()
            });
        }
        TuiClient {
            location: Location::Splash,
            selected_item: Some(0),
            items_len: 2,
            new_server_info: None,
            config,
        }
    }

    pub fn render(&mut self) -> Result<(), failure::Error> {
        // Terminal initialization
        let stdout = stdout().into_raw_mode()?;
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
                        if self.location == Location::Splash {
                            break;
                        } else {
                            self.selected_item = Some(0);
                            self.location = Location::Splash;
                        }
                    },
                    Key::Down => {
                        self.selected_item = if let Some(selected) = self.selected_item {
                            if selected >= self.items_len - 1 {
                                Some(0)
                            } else {
                                Some(selected + 1)
                            }
                        } else {
                            Some(0)
                        };
                    },
                    Key::Up => {
                        self.selected_item = if let Some(selected) = self.selected_item {
                            if selected > 0 {
                                Some(selected - 1)
                            } else {
                                Some(self.items_len - 1)
                            }
                        } else {
                            Some(0)
                        };
                    },
                    Key::Char('\n') => {
                        if self.location == Location::Splash {
                            let selected = self.selected_item.unwrap_or(0);
                            if selected == 0 {
                                self.new_server_info = Some(ServerInfo {
                                    name: String::new(),
                                    certificate: String::new(),
                                    hostname: String::new(),
                                });
                                self.location = Location::ConfigureServer;
                            }
                        } else if self.location == Location::ConfigureServer && self.selected_item == Some(3) {
                            if self.config.default_playername.is_empty() {
                                self.config.default_playername = self.new_server_info.clone().unwrap().name;
                            }
                            self.config.servers.push(self.new_server_info.clone().unwrap());
                            self.save_servers();
                            self.new_server_info = None;
                            self.selected_item = Some(0);
                            self.location = Location::Splash;
                        }
                    },
                    Key::Char('\t') => {
                        if self.location == Location::ConfigureServer {
                            self.selected_item = if let Some(selected) = self.selected_item {
                                if selected >= self.items_len - 1 {
                                    Some(0)
                                } else {
                                    Some(selected + 1)
                                }
                            } else {
                                Some(0)
                            };
                        }
                    }
                    Key::Char(c) => {
                        if self.location == Location::ConfigureServer {
                            match self.selected_item {
                                Some(0) => self.new_server_info.as_mut().unwrap().name.push(c),
                                Some(1) => self.new_server_info.as_mut().unwrap().hostname.push(c),
                                Some(2) => self.new_server_info.as_mut().unwrap().certificate.push(c),
                                _ => {}
                            }
                        }
                    },
                    Key::Backspace => {
                        if self.location == Location::ConfigureServer {
                            match self.selected_item {
                                Some(0) => { self.new_server_info.as_mut().unwrap().name.pop(); },
                                Some(1) => { self.new_server_info.as_mut().unwrap().hostname.pop(); },
                                Some(2) => { self.new_server_info.as_mut().unwrap().certificate.pop(); },
                                _ => {}
                            }
                        }
                    },
                    Key::Delete => {
                        let selection = self.selected_item.unwrap_or(0);
                        if self.location == Location::Splash && selection > 0 {
                            self.config.servers.remove(selection - 1);
                            self.selected_item = Some(0);
                            self.save_servers();
                        }
                    }
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
        // TODO paragraph
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
            .y_bounds([0.0, (size.height / 2) as f64])
            .render(&mut f, Rect::new(0, 0, size.width, size.height / 2));            
    }

    fn draw_server_list<B: tui::backend::Backend>(&mut self, mut f: &mut Frame<B>) {
        let size = f.size();

        let mut servers_list = vec!["Add a new server to the list"];
        for serv in &self.config.servers {
            servers_list.push(&*serv.hostname);
        }
        self.items_len = servers_list.len();

        let style = Style::default().fg(Color::Black).bg(Color::White);
        SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("Servers"))
                .items(&servers_list)
                .select(self.selected_item)
                .highlight_style(Style::default().fg(Color::LightGreen).modifier(Modifier::BOLD))
                .highlight_symbol(">")
                .render(&mut f, Rect::new(0, size.height / 2, size.width, size.height / 2));       
    }

    fn configure_new_server<B: tui::backend::Backend>(&mut self, mut f: &mut Frame<B>) {
        let size = f.size();

        let name = format!("Player name: {}\n", self.new_server_info.as_ref().unwrap().name);
        let hostname = format!("Hostname:    {}\n", self.new_server_info.as_ref().unwrap().hostname);
        let certificate = format!("Certificate: {}\n", self.new_server_info.as_ref().unwrap().certificate);

        let mut playing_text = vec![
            Text::styled(&name, if self.selected_item == Some(0) { Style::default().fg(Color::LightGreen).modifier(Modifier::BOLD) } else { Style::default() }),
            Text::styled(&hostname, if self.selected_item == Some(1) { Style::default().fg(Color::LightGreen).modifier(Modifier::BOLD) } else { Style::default() }),
            Text::styled(&certificate, if self.selected_item == Some(2) { Style::default().fg(Color::LightGreen).modifier(Modifier::BOLD) } else { Style::default() }),
            Text::styled("Save", if self.selected_item == Some(3) { Style::default().fg(Color::LightGreen).modifier(Modifier::BOLD) } else { Style::default() }),
        ];
        self.items_len = playing_text.len();

        Paragraph::new(playing_text.iter())
            .wrap(true)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Configure")
            )
            .render(f, Rect::new(0, size.height / 2, size.width, size.height / 2));
    }

    fn save_servers(&mut self) -> std::io::Result<()> {
        // TODO test if duplicates are ok?
        // TODO edit server
        // TODO check config
        let content = serde_json::to_string(&self.config)?;
        let mut file = File::create("config.json")?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}