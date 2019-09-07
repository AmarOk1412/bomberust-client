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

use std::sync::{Arc, Mutex};
use std::io::{stdin,stdout,Write};
use serde::Serialize;
use rmps::Serializer;

use super::super::net::msg::*;

pub struct KeyHandler {
    pub send_buf: Arc<Mutex<Option<Vec<u8>>>>,
}

impl KeyHandler {
    pub fn new(send_buf: Arc<Mutex<Option<Vec<u8>>>>) -> KeyHandler {
        KeyHandler {
            send_buf
        }
    }

    fn clean_string(string: String) -> String {
        let mut s = string.clone();
        if let Some('\n') = s.chars().next_back() {
            s.pop();
        }
        if let Some('\r') = s.chars().next_back() {
            s.pop();
        }
        s
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

    fn print_help() {
        println!("Possible commands (outside a game):");
        println!(" c          create a new room");
        println!(" j [room]   join a room");
        println!(" l          leave the current room");
        println!(" g          start a new game");
        println!("");
        println!("Possible commands (in game):");
        println!("Send A,W,S,D to move or SPACE to put a bomb");
    }

    pub fn run(&mut self) {
        println!("WELCOME TO BOMBER RUST v0.0!");
        let mut s = String::new();
        print!("Player name: ");
        let mut buf = Vec::new();
        while s.is_empty() {
            let _ = stdout().flush();
            stdin().read_line(&mut s).expect("Did not enter a correct string");
            s = KeyHandler::clean_string(s);
            if !s.is_empty() {
                let msg = PlayerMsg::new(s.clone());
                msg.serialize(&mut Serializer::new(&mut buf)).unwrap();
                self.send_rtp(&mut buf);
            }
        }
        s = String::new();
        loop {
            KeyHandler::print_help();
            let _ = stdout().flush();
            stdin().read_line(&mut s).expect("Did not enter a correct string");
            s = KeyHandler::clean_string(s);
            if !s.is_empty() {
                if s == "c" {
                    let msg = Msg::new(String::from("create"));
                    msg.serialize(&mut Serializer::new(&mut buf)).unwrap();
                } else if s == "l" {
                    let msg = Msg::new(String::from("leave"));
                    msg.serialize(&mut Serializer::new(&mut buf)).unwrap();
                } else if s == "g" {
                    let msg = Msg::new(String::from("launch"));
                    msg.serialize(&mut Serializer::new(&mut buf)).unwrap();
                } else if s == " " {
                    let msg = Msg::new(String::from("bomb"));
                    msg.serialize(&mut Serializer::new(&mut buf)).unwrap();
                } else if s.starts_with("j") {
                    let room: u64 = String::from(&s[2..]).parse().unwrap_or(0);
                    let msg = JoinMsg::new(room);
                    msg.serialize(&mut Serializer::new(&mut buf)).unwrap();
                }
                self.send_rtp(&mut buf);
                s = String::new();
            }
        }
    }
}