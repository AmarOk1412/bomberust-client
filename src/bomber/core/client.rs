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

use crate::bomber::net::msg::*;

use futures::sync::mpsc;
use rmps::Deserializer;
use rmps::decode::Error;
use serde::Deserialize;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use crate::bomber::net::diff_msg::*;
use crate::bomber::gen::map::Map;
use crate::bomber::gen::item::*;

pub struct RtpBuf {
    data: [u8; 65536],
    size: u16,
    wanted: u16,
}

pub struct Client
{
    pub send_buf: Arc<Mutex<Option<Vec<u8>>>>,
    pub tx: mpsc::Sender<u8>,
    pub rtp_buf: RtpBuf,
    pub map: Option<Map>
}

impl Client {
    pub fn new(send_buf: Arc<Mutex<Option<Vec<u8>>>>, tx: mpsc::Sender<u8>) -> Client {
        Client {
            tx,
            send_buf: send_buf,
            rtp_buf: RtpBuf {
                data: [0; 65536],
                size: 0,
                wanted: 0,
            },
            map: None
        }
    }

    pub fn move_player(&mut self, diff: PlayerMove) {
        let map = self.map.as_mut().unwrap();
        let player = &mut map.players[diff.id as usize];
        player.x = diff.x;
        player.y = diff.y;
        println!("{}", map);
    }

    pub fn player_put_bomb(&mut self, diff: PlayerPutBomb) {
        let map = self.map.as_mut().unwrap();
        let item = &mut map.items[diff.x + diff.y * map.w];
        *item = Some(Box::new(bomb::BombItem {}));
        println!("{}", map);
    }

    pub fn player_die(&mut self, diff: PlayerDie) {
        let map = self.map.as_mut().unwrap();
        map.players.remove(diff.id as usize);
        println!("{}", map);
    }

    pub fn parse_rtp(&mut self, pkt: Vec<u8>) {
        info!("rx:{}", pkt.len());
        let cur = Cursor::new(&*pkt);
        let mut de = Deserializer::new(cur);
        let actual: Result<Msg, Error> = Deserialize::deserialize(&mut de);
        if actual.is_ok() {
            let msg_type = actual.unwrap().msg_type;
            let cur = Cursor::new(&*pkt);
            let mut de = Deserializer::new(cur);
            if msg_type == "map" {
                let msg: MapMsg = Deserialize::deserialize(&mut de).unwrap();
                println!("{}", msg.map);
                self.map = Some(msg.map);
            } else if msg_type == "player_move_diff" {
                let msg: PlayerMove = Deserialize::deserialize(&mut de).unwrap();
                self.move_player(msg);
            } else if msg_type == "player_put_bomb_diff" {
                let msg: PlayerPutBomb = Deserialize::deserialize(&mut de).unwrap();
                self.player_put_bomb(msg);
            } else if msg_type == "player_die" {
                let msg: PlayerDie = Deserialize::deserialize(&mut de).unwrap();
                self.player_die(msg);
            }
        }
    }

    pub fn process_rx(&mut self, buf: &Vec<u8>) {
        let mut pkts: Vec<Vec<u8>> = Vec::new();
        let rtp_buf = &mut self.rtp_buf;
        let size = buf.len() as u16;
        let mut parsed = 0;
        loop {
            let mut pkt_len = size - parsed;
            let mut store_remaining = true;
            let mut start = parsed;

            if rtp_buf.size != 0 || rtp_buf.wanted != 0 {
                // There is a packet to complete
                if rtp_buf.size == 1 {
                    pkt_len = ((rtp_buf.data[0] as u16) << 8) + buf[0] as u16;
                    rtp_buf.size = 0; // The packet is eaten
                    parsed += 1;
                    start += 1;
                    if pkt_len + parsed <= size {
                        store_remaining = false;
                        parsed += size;
                    } else {
                        rtp_buf.wanted = pkt_len;
                    }
                } else if pkt_len + rtp_buf.size >= rtp_buf.wanted {
                    // We have enough data to build the new packet to parse
                    store_remaining = false;
                    let eaten_bytes = rtp_buf.wanted - rtp_buf.size;
                    rtp_buf.data[rtp_buf.size as usize..]
                        .copy_from_slice(&buf[(parsed as usize)..(parsed as usize + eaten_bytes as usize)]);
                    pkt_len = rtp_buf.wanted;
                    parsed += eaten_bytes;
                    rtp_buf.size = 0;
                    rtp_buf.wanted = 0;
                }
            } else if pkt_len > 1 {
                pkt_len = ((buf[0] as u16) << 8) + buf[1] as u16;
                parsed += 2;
                start += 2;
                if pkt_len + parsed <= size {
                    store_remaining = false;
                    parsed += pkt_len;
                } else {
                    rtp_buf.wanted = pkt_len;
                }
            }
            if store_remaining {
                let stored_size = size - parsed;
                rtp_buf.data[rtp_buf.size as usize..]
                    .copy_from_slice(&buf[(parsed as usize)..(parsed as usize + stored_size as usize)]);
                rtp_buf.size += stored_size;
                break;
            }

            let pkt = buf[(start as usize)..(start as usize + pkt_len as usize)].to_vec();
            pkts.push(pkt);
            if parsed >= size {
                break;
            }
        }
        for pkt in pkts {
            self.parse_rtp(pkt);
        }
    }
}