use crate::bomber::core::Client;
use crate::bomber::net::{ ConnectionState, TlsClient, TlsClientConfig };
use crate::bomber::net::msg::*;
use crate::bomber::gen::item::*;
use crate::bomber::gen::utils::*;
use crate::util::{ Config, Event, Events };

use futures::sync::mpsc;
use rmps::Serializer;
use serde::Serialize;
use std::fs::{ self, File };
use std::io::{ stdout, Write };
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};
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
    pub address: String,
    pub certificate: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientConfig {
    servers: Vec<ServerInfo>,
    default_playername: String,
}

// TODO separate by layout
pub struct TuiClient {
    location: Location,
    selected_item: Option<usize>,
    items_len: usize,
    new_server_info: Option<ServerInfo>,
    config: ClientConfig,
    client_thread: Option<thread::JoinHandle<()>>,
    connected_item: Option<String>,
    server_state: Arc<Mutex<Option<ConnectionState>>>,
    last_error: String,
    room_to_join: String,
    send_buf: Arc<Mutex<Option<Vec<u8>>>>,
    client: Option<Arc<Mutex<Client>>>,
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
            client_thread: None,
            connected_item: None,
            server_state: Arc::new(Mutex::new(None)),
            last_error: String::new(),
            send_buf: Arc::new(Mutex::new(None)),
            room_to_join: String::new(),
            client: None,
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
                        self.draw_servers_list(&mut f);
                    },
                    Location::ConfigureServer => {
                        self.render_splash(&mut f);
                        self.configure_new_server(&mut f);
                    },
                    Location::Lobby => {
                        self.render_splash(&mut f);
                        self.draw_rooms_list(&mut f);
                    },
                    Location::Room => {
                        self.render_splash(&mut f);
                        self.draw_room_view(&mut f);
                    },
                    Location::Game => {
                        self.render_game(&mut f);
                    },
                    _ => { }
                }
            });

            match self.location {
                Location::Splash => {
                    if !self.events_splash(&events) {
                        break;
                    }
                    if self.server_state.lock().unwrap().is_some() {
                        // TODO partialeq
                        let formatted = format!("{}", *self.server_state.lock().unwrap().as_ref().unwrap());
                        if formatted == "Connected" {
                            self.selected_item = Some(0);
                            self.room_to_join = String::new();
                            self.location = Location::Lobby;
                        }
                    }
                },
                Location::ConfigureServer => {
                    if !self.events_splash(&events) {
                        break;
                    }
                },
                Location::Lobby => {
                    if !self.events_lobby(&events) {
                        break;
                    }
                },
                Location::Room => {
                    if !self.events_in_room(&events) {
                        break;
                    }
                },
                Location::Game => {
                    if !self.events_in_game(&events) {
                        break;
                    }
                },
                _ => { }
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


    fn render_game<B: tui::backend::Backend>(&mut self, mut f: &mut Frame<B>) {
        let size = f.size();

        Canvas::default()
            .block(Block::default().borders(Borders::ALL).title("Bomberust"))
            .paint(|ctx| {})
            .render(&mut f, Rect::new(0, 0, size.width, size.height));  

        let client_map = self.client.as_ref().unwrap().lock().unwrap().map.as_ref().unwrap().clone();
        let square_size = 3;
        let offset_x = std::cmp::max(0, (size.width as usize - client_map.w * square_size) / 2);
        let offset_y = std::cmp::max(0, (size.height as usize - client_map.h * square_size) / 2);

        // TODO get from config
        let players = ["🐧", "🐥", "🦂", "🐙"];
        let mut player_idx = 0;

        for p in &client_map.players {
            if p.dead {
                player_idx += 1;
                continue;
            }

            let mut x = (p.x * square_size as f32);
            if x < 0.0 {
                x = 0.0;
            }
            let mut y = (p.y * square_size as f32);
            if y < 0.0 {
                y = 0.0;
            }

            let rect = Rect::new((x + offset_x as f32) as u16, (y + offset_y as f32) as u16, square_size as u16, square_size as u16);
            let mut player = vec![Text::raw(players[player_idx])];
            Paragraph::new(player.iter())
                .wrap(true)
                .block(Block::default().borders(Borders::NONE))
                .render(&mut f, rect);
            
            player_idx += 1;
        }

        for x in 0..client_map.w {
            for y in 0..client_map.h {
                let pos = x + client_map.w * y;
                let sq = client_map.squares[pos];

                let rect = Rect::new((x * square_size + offset_x) as u16, (y * square_size + offset_y) as u16, square_size as u16, square_size as u16);

                match sq.sq_type {
                    SquareType::Water => {
                        let mut water = vec![Text::raw("~.~"), Text::raw(".~."), Text::raw("~.~")];
                        Paragraph::new(water.iter())
                            .wrap(true)
                            .block(Block::default().borders(Borders::NONE))
                            .render(&mut f, rect);
                        Canvas::default()
                        .block(Block::default().borders(Borders::NONE).style(Style::default().bg(Color::Blue)))
                        .paint(|ctx| { })
                        .render(&mut f, rect);
                    },
                    SquareType::Block => {
                        Canvas::default()
                        .block(Block::default().borders(Borders::NONE).style(Style::default().bg(Color::Black)))
                        .paint(|ctx| { })
                        .render(&mut f, rect);
                    },
                    SquareType::Empty => {
                        
                        match &client_map.items[pos] {
                            Some(i) => {
                                if i.name() == "DestructibleBox" {
                                    Canvas::default()
                                    .block(Block::default().borders(Borders::NONE).style(Style::default().bg(Color::Rgb(55,27,0))))
                                    .paint(|ctx| { })
                                    .render(&mut f, rect);
                                } else {        
                                    if i.name() == "Bomb" {
                                        let mut bomb = vec![Text::raw("💣")];
                                        Paragraph::new(bomb.iter())
                                            .wrap(true)
                                            .block(Block::default().borders(Borders::NONE))
                                            .render(&mut f, rect);
                                    } else if i.name() == "Bonus" {
                                        let mut bonus = vec![Text::raw("🌟")];
                                        Paragraph::new(bonus.iter())
                                            .wrap(true)
                                            .block(Block::default().borders(Borders::NONE))
                                            .render(&mut f, rect);
                                    } else if i.name() == "Malus" {
                                        let mut malus = vec![Text::raw("💀")];
                                        Paragraph::new(malus.iter())
                                            .wrap(true)
                                            .block(Block::default().borders(Borders::NONE))
                                            .render(&mut f, rect);
                                    }

                                    Canvas::default()
                                    .block(Block::default().borders(Borders::NONE).style(Style::default().bg(Color::Yellow)))
                                    .paint(|ctx| {})
                                    .render(&mut f, rect);
                                }
                            }
                            _ => {
                                Canvas::default()
                                .block(Block::default().borders(Borders::NONE).style(Style::default().bg(Color::Yellow)))
                                .paint(|ctx| {})
                                .render(&mut f, rect);
                            }
                        }
                        
                    },
                    SquareType::Wall(d) => {
                        match d {
                            crate::bomber::gen::utils::Direction::North => {
                                let mut wall = vec![Text::raw("╔═╗")];
                                Paragraph::new(wall.iter())
                                    .wrap(true)
                                    .block(Block::default().borders(Borders::NONE))
                                    .render(&mut f, rect);
                            },
                            crate::bomber::gen::utils::Direction::South => {
                                let rect = Rect::new((x * square_size + offset_x) as u16, (y * square_size + square_size - 1 + offset_y) as u16, square_size as u16, 1);
                                let mut wall = vec![Text::raw("╚═╝")];
                                Paragraph::new(wall.iter())
                                    .wrap(true)
                                    .block(Block::default().borders(Borders::NONE))
                                    .render(&mut f, rect);
                            },
                            crate::bomber::gen::utils::Direction::West => {
                                let mut wall = vec![Text::raw("╔\n║\n╚")];
                                Paragraph::new(wall.iter())
                                    .wrap(true)
                                    .block(Block::default().borders(Borders::NONE))
                                    .render(&mut f, rect);
                            },
                            crate::bomber::gen::utils::Direction::East => {
                                let rect = Rect::new((x * square_size + square_size - 1 + offset_x) as u16, (y * square_size + offset_y) as u16, 1, square_size as u16);
                                let mut wall = vec![Text::raw("╗\n║\n╝")];
                                Paragraph::new(wall.iter())
                                    .wrap(true)
                                    .block(Block::default().borders(Borders::NONE))
                                    .render(&mut f, rect);
                            },
                        }
                        Canvas::default()
                        .block(Block::default().borders(Borders::NONE).style(Style::default().bg(Color::Yellow)))
                        .paint(|ctx| {})
                        .render(&mut f, rect);
                    }
                }
            }
        }
               
    }

    fn draw_servers_list<B: tui::backend::Backend>(&mut self, mut f: &mut Frame<B>) {
        let size = f.size();

        let mut servers_list = vec!["Add a new server to the list"];
        for serv in &self.config.servers {
            servers_list.push(&*serv.address);
        }
        self.items_len = servers_list.len();

        let title = match &self.connected_item {
            Some(s) => format!("Servers - {} ({})", s, *self.server_state.lock().unwrap().as_ref().unwrap()),
            None => String::from("Servers")
        };

        // TODO title style when connecting
        SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title(&*title))
                .items(&servers_list)
                .select(self.selected_item)
                .highlight_style(Style::default().fg(Color::LightGreen).modifier(Modifier::BOLD))
                .highlight_symbol(">")
                .render(&mut f, Rect::new(0, size.height / 2, size.width, size.height / 2));       
    }

    fn draw_room_view<B: tui::backend::Backend>(&mut self, mut f: &mut Frame<B>) {
        let size = f.size();

        let now = SystemTime::now();
        let odd_sec = now.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() % 2 == 0;
        let color = if odd_sec { Color::Yellow } else { Color::Rgb(225, 125, 0) };
        let style = Style::default().fg(color);

        Canvas::default()
            .block(
                Block::default()
                .borders(Borders::ALL).title("Room configuration")
                .border_style(style).title_style(style)
            ).paint(|ctx| {
                // TODO
                ctx.print(0.0, (size.height / 4) as f64, "=> Not available for now", Color::Yellow);
                ctx.print((size.width / 6) as f64 - 1.5, 1.0, "GO!", Color::White);
            })
            .x_bounds([0.0, (size.width / 3) as f64])
            .y_bounds([0.0, (size.height / 2) as f64])
            .render(&mut f, Rect::new(0, size.height / 2, size.width / 3, size.height / 2));

        Canvas::default()
            .paint(|ctx| {})
            .block(Block::default().borders(Borders::NONE).style(Style::default().bg(color)))
            .render(&mut f, Rect::new(1, size.height - 4, size.width / 3 - 2, 3));

        Canvas::default()
            .block(Block::default().borders(Borders::ALL).title("Players"))
            .paint(|ctx| {
                ctx.print(0.0, (size.height / 4) as f64, "=> Not available for now", Color::Yellow);
                // TODO
            })
            .x_bounds([0.0, size.width as f64])
            .y_bounds([0.0, (size.height / 2) as f64])
            .render(&mut f, Rect::new(size.width / 3, size.height / 2, size.width / 3, size.height / 2));

        Canvas::default()
            .block(Block::default().borders(Borders::ALL).title("Chat"))
            .paint(|ctx| {
                ctx.print(0.0, (size.height / 4) as f64, "=> Not available for now", Color::Yellow);
                // TODO
            })
            .x_bounds([0.0, size.width as f64])
            .y_bounds([0.0, (size.height / 2) as f64])
            .render(&mut f, Rect::new(2 * size.width / 3, size.height / 2, size.width / 3, size.height / 2));
    }

    fn draw_rooms_list<B: tui::backend::Backend>(&mut self, mut f: &mut Frame<B>) {
        let size = f.size();

        let new_room = format!("{} Create a new room\n", if self.selected_item == Some(0) { ">" } else { "-" });
        let join_room = format!("{} Join a room (type ID and press Enter): {}\n", if self.selected_item == Some(1) { ">" } else { "-" }, self.room_to_join);

        let mut rooms_list = vec![
            Text::styled(&new_room, if self.selected_item == Some(0) { Style::default().fg(Color::LightGreen).modifier(Modifier::BOLD) } else { Style::default() }),
            Text::styled(&join_room, if self.selected_item == Some(1) { Style::default().fg(Color::LightGreen).modifier(Modifier::BOLD) } else { Style::default() }),
        ];
        self.items_len = rooms_list.len();

        Paragraph::new(rooms_list.iter())
            .wrap(true)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).title("Rooms"))
            .render(&mut f, Rect::new(0, size.height / 2, size.width, size.height / 2));
    }

    

    fn configure_new_server<B: tui::backend::Backend>(&mut self, mut f: &mut Frame<B>) {
        let size = f.size();

        let name = format!("Player name: {}\n", self.new_server_info.as_ref().unwrap().name);
        let address = format!("Address:     {}\n", self.new_server_info.as_ref().unwrap().address);
        let certificate = format!("Certificate: {}\n", self.new_server_info.as_ref().unwrap().certificate);
        let error = format!("\n{}\n", self.last_error);

        let mut playing_text = vec![
            Text::styled(&name, if self.selected_item == Some(0) { Style::default().fg(Color::LightGreen).modifier(Modifier::BOLD) } else { Style::default() }),
            Text::styled(&address, if self.selected_item == Some(1) { Style::default().fg(Color::LightGreen).modifier(Modifier::BOLD) } else { Style::default() }),
            Text::styled(&certificate, if self.selected_item == Some(2) { Style::default().fg(Color::LightGreen).modifier(Modifier::BOLD) } else { Style::default() }),
            Text::styled("Save", if self.selected_item == Some(3) { Style::default().fg(Color::LightGreen).modifier(Modifier::BOLD) } else { Style::default() }),
            Text::styled(&error, Style::default().fg(Color::Red).modifier(Modifier::BOLD)),
        ];
        self.items_len = playing_text.len() - 1;

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
        let content = serde_json::to_string(&self.config)?;
        let mut file = File::create("config.json")?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    fn connect_server(&mut self, server_idx: usize) {
        if self.config.servers.len() < server_idx || self.client_thread.is_some() {
            if *self.server_state.lock().unwrap() != Some(ConnectionState::Disconnected) {
                return;
            }
        }

        let server = self.config.servers.get(server_idx).unwrap().clone();
        let (tx, rx) = mpsc::channel::<u8>(65536);
        *self.server_state.lock().unwrap() = Some(ConnectionState::Connecting);
        let server_state = self.server_state.clone();
        self.connected_item = Some(server.address.clone());

        let client = Arc::new(Mutex::new(Client::new(self.send_buf.clone(), tx)));
        let client_cloned = client.clone();
        self.client = Some(client);
        self.client_thread = Some(thread::spawn(move || {
            let config = TlsClientConfig {
                server_state,
                addr: server.address.clone(),
                cert: server.certificate.clone(),
                client: client_cloned,
            };
            TlsClient::start(&config);
        }));
    }

    fn events_splash(&mut self, events: &Events) -> bool {
        // TODO split in functions
        let events = events.next();
        if !events.is_ok() {
            return true;
        }
        match events.unwrap() {
            Event::Input(input) => match input {
                Key::Esc => {
                    if self.location == Location::Splash {
                        return false;
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
                                name: self.config.default_playername.clone(),
                                certificate: String::new(),
                                address: String::new(),
                            });
                            self.last_error = String::new();
                            self.location = Location::ConfigureServer;
                        } else {
                            self.connect_server(selected - 1);
                        }
                    } else if self.location == Location::ConfigureServer && self.selected_item == Some(3) {
                        let new_server = self.new_server_info.clone().unwrap();
                        if new_server.name.is_empty() {
                            self.last_error = String::from("Please enter your player name");
                            return true;
                        }
                        if new_server.address.is_empty() {
                            self.last_error = String::from("Please provide a server address");
                            return true;
                        } else {
                            let server: Result<SocketAddr, _> = new_server.address.parse();
                            if !server.is_ok() {
                                self.last_error = String::from("Incorrect server address");
                                return true;
                            }
                        }
                        if self.config.servers.len() == 0 {
                            self.config.default_playername = new_server.name.clone();
                        }

                        let mut add_server = true;
                        for serv in &self.config.servers {
                            if serv.address == new_server.address {
                                add_server = false;
                                break;
                            }
                        }
                        if add_server {
                            self.config.servers.push(new_server);
                            self.save_servers();
                        }
                        self.new_server_info = None;
                        self.selected_item = Some(0);
                        self.location = Location::Splash;
                    } else if self.location == Location::ConfigureServer {
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
                            Some(1) => self.new_server_info.as_mut().unwrap().address.push(c),
                            Some(2) => self.new_server_info.as_mut().unwrap().certificate.push(c),
                            _ => {}
                        }
                    }
                },
                Key::Backspace => {
                    if self.location == Location::ConfigureServer {
                        match self.selected_item {
                            Some(0) => { self.new_server_info.as_mut().unwrap().name.pop(); },
                            Some(1) => { self.new_server_info.as_mut().unwrap().address.pop(); },
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
        true
    }

    fn events_lobby(&mut self, events: &Events) -> bool {
        let events = events.next();
        if !events.is_ok() {
            return true;
        }
        match events.unwrap() {
            Event::Input(input) => match input {
                Key::Esc => {
                    return false;
                    //self.selected_item = Some(0);
                    //self.location = Location::Splash;
                    // TODO return to splash + close connection
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
                    let mut buf = Vec::new();
                    if self.selected_item == Some(0) {
                        let msg = Msg::new(String::from("create"));
                        msg.serialize(&mut Serializer::new(&mut buf)).unwrap();
                        self.send_rtp(&mut buf);
                        // TODO server should send some messages
                        self.location = Location::Room;
                        self.selected_item = Some(0);
                    } else if self.selected_item == Some(1) {
                        let room: u64 = self.room_to_join.parse().unwrap_or(0);
                        let msg = JoinMsg::new(room);
                        msg.serialize(&mut Serializer::new(&mut buf)).unwrap();
                        self.send_rtp(&mut buf);
                        if room > 0 {
                            // TODO server should send some messages
                            self.location = Location::Room;
                            self.selected_item = Some(0);
                        }
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
                },
                Key::Char(c) => {
                    self.room_to_join.push(c);
                },
                Key::Backspace => {
                    self.room_to_join.pop();
                },
                _ => {}
            },
            Event::Tick => {
            }
        }
        true
    }

    fn events_in_room(&mut self, events: &Events) -> bool {
        let events = events.next();
        if !events.is_ok() {
            return true;
        }
        match events.unwrap() {
            Event::Input(input) => match input {
                Key::Esc => {
                    let mut buf = Vec::new();
                    let msg = Msg::new(String::from("leave"));
                    msg.serialize(&mut Serializer::new(&mut buf)).unwrap();
                    // TODO get event from server
                    self.location = Location::Lobby;
                    self.room_to_join = String::new();
                },
                Key::Char('\n') => {
                    if self.selected_item == Some(0) {
                        let mut buf = Vec::new();
                        let msg = Msg::new(String::from("launch"));
                        msg.serialize(&mut Serializer::new(&mut buf)).unwrap();
                        self.send_rtp(&mut buf);
                        // TODO server should send some messages
                        self.location = Location::Game;
                    }
                },
                _ => {}
            },
            Event::Tick => {
            }
        }
        true
    }

    fn events_in_game(&mut self, events: &Events) -> bool {
        let events = events.next();
        if !events.is_ok() {
            return true;
        }
        let mut buf = Vec::new();
        match events.unwrap() {
            Event::Input(input) => match input {
                Key::Esc => {
                    // TODO
                },
                Key::Char('w') => {
                    let msg = MoveMsg::new(crate::bomber::gen::utils::Direction::North);
                    msg.serialize(&mut Serializer::new(&mut buf)).unwrap();
                    self.send_rtp(&mut buf);
                },
                Key::Char('a') => {
                    let msg = MoveMsg::new(crate::bomber::gen::utils::Direction::West);
                    msg.serialize(&mut Serializer::new(&mut buf)).unwrap();
                    self.send_rtp(&mut buf);
                },
                Key::Char('s') => {
                    let msg = MoveMsg::new(crate::bomber::gen::utils::Direction::South);
                    msg.serialize(&mut Serializer::new(&mut buf)).unwrap();
                    self.send_rtp(&mut buf);
                },
                Key::Char('d') => {
                    let msg = MoveMsg::new(crate::bomber::gen::utils::Direction::East);
                    msg.serialize(&mut Serializer::new(&mut buf)).unwrap();
                    self.send_rtp(&mut buf);
                },
                Key::Char(' ') => {
                    let msg = Msg::new(String::from("bomb"));
                    msg.serialize(&mut Serializer::new(&mut buf)).unwrap();
                    self.send_rtp(&mut buf);
                },
                _ => {}
            },
            Event::Tick => {
            }
        }
        true
    }

    fn send_rtp(&mut self, send: &mut Vec<u8>) {
        if send.len() > (2 as usize).pow(16) {
            error!("Can't send RTP packet because buffer is too long");
            return;
        }
        let len = send.len() as u16;
        let mut send_buf : Vec<u8> = Vec::with_capacity(65536);
        send_buf.push((len >> 8) as u8);
        send_buf.push((len as u16 % (2 as u16).pow(8)) as u8);
        send_buf.append(send);
        *self.send_buf.lock().unwrap() = Some(send_buf);
    }
}