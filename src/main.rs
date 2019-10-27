/**
 * Copyright (c) 2019, Sébastien Blin <sebastien.blin@enconn.fr>
 * All rights reserved.
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 * * Redistributions of source code must retain the above copyright
 *  notice, this list of conditions and the following disclaimer.
 * * Redistributions in binary form must reproduce the above copyright
 *  notice, this list of conditions and the following disclaimer in the
 *  documentation and/or other materials provided with the distribution.
 * * Neither the name of the University of California, Berkeley nor the
 *  names of its contributors may be used to endorse or promote products
 *  derived from this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE REGENTS AND CONTRIBUTORS ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
 * WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL THE REGENTS AND CONTRIBUTORS BE LIABLE FOR ANY
 * DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
 * (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
 * LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
 * ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
 * SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 **/


extern crate env_logger;
extern crate futures;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate rmp_serde as rmps;
extern crate tokio;
extern crate tokio_rustls;
extern crate tokio_stdin_stdout;
extern crate typetag;
extern crate webpki;
extern crate webpki_roots;

mod bomber;

use bomber::core::{Client, KeyHandler, TuiClient};
use bomber::net::{TlsClient, TlsClientConfig};

use std::sync::{Arc, Mutex};
use std::thread;
use tui::widgets::canvas::Line;
use tui::widgets::canvas::Points;
use futures::sync::mpsc;


extern crate failure;
extern crate tui;
extern crate termion;


#[allow(dead_code)]
mod util;

/*use std::io;
use std::time::Duration;



use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Style, Color};
use tui::widgets::canvas::{Canvas, Map, MapResolution, Rectangle};
use tui::widgets::{Block, Borders, Widget};
use tui::Terminal;

use util::{Config, Event, Events};

struct App {
    player: Rect,
    game_x: u16,
    game_y: u16,
    game_h: u16,
    game_w: u16,
}

impl App {
    fn new() -> App {
        App {
            player: Rect::new(0, 0, 0, 0),
            game_x: 0,
            game_y: 0,
            game_h: 0,
            game_w: 0,
        }
    }

    fn update(&mut self) {

        if self.player.y > self.game_y + self.game_h - self.player.height {
            self.player.y = self.game_y + self.game_h - self.player.height;
        } else if self.player.y < self.game_y {
            self.player.y = self.game_y;
        }

        if self.player.x > self.game_x + self.game_w - self.player.width {
            self.player.x = self.game_x + self.game_w - self.player.width;
        } else if self.player.x < self.game_x {
            self.player.x = self.game_x;
        }
    }
}*/

fn main() {
    let mut client = TuiClient::new();
    client.render();
    // Terminal initialization
    /*let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // Setup event handlers
    let config = Config {
        tick_rate: Duration::from_millis(100),
        ..Default::default()
    };
    let events = Events::with_config(config);

    // App
    let mut app = App::new();

    loop {
        terminal.draw(|mut f| {
            let size = f.size();
            let w = size.width / (23 + 2);
            let h = size.height / (13 + 2);
            let square_size = std::cmp::min(w, h);
            let offset_x = ((((size.width as f32 / square_size as f32) - 23 as f32)  * square_size as f32) / 2.0) as u16;
            let offset_y = ((((size.height as f32 / square_size as f32) - 13 as f32)  * square_size as f32) / 2.0) as u16;

            Canvas::default()
                .block(Block::default().borders(Borders::ALL).title("Bomberust"))
                .paint(|ctx| {
                })
                .x_bounds([0.0, size.width as f64])
                .y_bounds([0.0, size.height as f64])
                .render(&mut f, size);

            Canvas::default()
                .paint(|ctx| {})
                .block(Block::default().borders(Borders::NONE).style(Style::default().bg(Color::White)))
                .render(&mut f, Rect::new(offset_x - square_size,
                                        offset_y - square_size,
                                        (23 + 2) * square_size,
                                        (13 + 2) * square_size));
            // TODO size
            for x in 0..23 {
                for y in 0..13 {
                    let mut color = Color::Yellow;
                    if x % 2 == 1 && y % 2 == 1 {
                        color = Color::Black;
                    } 
                    Canvas::default()
                        .paint(|ctx| {})
                        .block(Block::default().borders(Borders::NONE).style(Style::default().bg(color)))
                        .render(&mut f, Rect::new(x * square_size + offset_x, y * square_size + offset_y, square_size, square_size));
                }
            }

            app.player.width = square_size;//std::cmp::max(2, square_size);
            app.player.height = square_size;//std::cmp::max(2, square_size);
            app.game_x = offset_x;
            app.game_y = offset_y;
            app.game_w = 23 * square_size;
            app.game_h = 13 * square_size;

            Canvas::default()
                .block(Block::default().borders(Borders::NONE).style(Style::default().bg(Color::LightBlue)))
                .paint(|ctx| {
                    //ctx.print(0.0, 0.0, "\n 1 ", Color::Yellow);
                })
                .render(&mut f, app.player);


            Canvas::default()
                .block(Block::default().borders(Borders::NONE).style(Style::default().bg(Color::Gray)))
                .paint(|ctx| { })
                .x_bounds([0.0, 100.0])
                .y_bounds([0.0, 100.0])
                .render(&mut f, Rect::new(0 * square_size + offset_x, 1 * square_size + offset_y, square_size, square_size));

        })?;


        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                Key::Down => {
                    if app.player.y < app.game_y + app.game_h - app.player.height {
                        app.player.y += 1;
                    }
                }
                Key::Up => {
                    if app.player.y > app.game_y {
                        app.player.y -= 1;
                    }
                }
                Key::Right => {
                    if app.player.x < app.game_x + app.game_w - app.player.width {
                        app.player.x += 1;
                    }
                }
                Key::Left => {
                    if app.player.x > app.game_x {
                        app.player.x -= 1;
                    }
                }

                _ => {}
            },
            Event::Tick => {
                app.update();
            }
        }
    }

    Ok(())*/
}



/*
fn main() {
    // Init logging
    env_logger::init();

    let send_buf: Arc<Mutex<Option<Vec<u8>>>> = Arc::new(Mutex::new(None));
    let send_buf_cloned = send_buf.clone();
    let (tx, rx) = mpsc::channel::<u8>(65536);

    let client = Arc::new(Mutex::new(Client::new(send_buf, tx)));
    let client_cloned = client.clone();
    let client_thread = thread::spawn(move || {
        let config = TlsClientConfig {
            host : String::from("127.0.0.1"),
            port : 2542,
            cert : String::from("./ca.cert"),
            client: client_cloned,
        };
        TlsClient::start(&config);
    });

    let key_handler_thread = thread::spawn(move || {
        let mut key_handler = KeyHandler::new(send_buf_cloned);
        key_handler.run();
    });

    let _ = client_thread.join();
    let _ = key_handler_thread.join();
}*/