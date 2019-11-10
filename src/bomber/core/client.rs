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
    pub map: Option<Map>,
    pub linked_id: Option<u64>,
    pub current_room_id: Option<u64>
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
            map: None,
            linked_id: None,
            current_room_id: None,
        }
    }

    fn move_player(&mut self, diff: PlayerMove) {
        let map = self.map.as_mut().unwrap();
        let player = &mut map.players[diff.id as usize];
        player.x = diff.x;
        player.y = diff.y;
    }

    fn player_put_bomb(&mut self, diff: PlayerPutBomb) {
        let map = self.map.as_mut().unwrap();
        let item = &mut map.items[diff.x + diff.y * map.w];
        *item = Some(Box::new(bomb::BombItem {}));
    }

    fn bomb_explode(&mut self, diff: BombExplode) {
        let map = self.map.as_mut().unwrap();
        let item = &mut map.items[diff.w as usize + diff.h as usize * map.w];
        *item = None;
    }

    fn create_item(&mut self, diff: CreateItem) {
        let map = self.map.as_mut().unwrap();
        let item = &mut map.items[diff.w as usize + diff.h as usize * map.w];
        *item = diff.item;
    }

    fn destroy_item(&mut self, diff: DestroyItem) {
        let map = self.map.as_mut().unwrap();
        let item = &mut map.items[diff.w as usize + diff.h as usize * map.w];
        *item = None;
    }

    fn player_die(&mut self, diff: PlayerDie) {
        let map = self.map.as_mut().unwrap();
        if diff.id < map.players.len() as u64 {
            map.players[diff.id as usize].dead = true;
            if self.linked_id.is_some() && self.linked_id.unwrap() == diff.id {
                // TODO! YOU DIED
            }
        }
    }

    fn update_square(&mut self, diff: UpdateSquare) {
        let map = self.map.as_mut().unwrap();
        let square = &mut map.squares[diff.x as usize + diff.y as usize * map.w];
        square.sq_type = diff.square;
    }

    pub fn parse_rtp(&mut self, pkt: Vec<u8>) {
        let cur = Cursor::new(&*pkt);
        let mut de = Deserializer::new(cur);
        let actual: Result<Msg, Error> = Deserialize::deserialize(&mut de);
        if actual.is_ok() {
            let msg_type = actual.unwrap().msg_type;
            let cur = Cursor::new(&*pkt);
            let mut de = Deserializer::new(cur);
            info!("RX {}", msg_type);
            if msg_type == "map" {
                let msg: MapMsg = Deserialize::deserialize(&mut de).unwrap();
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
            } else if msg_type == "bomb_explode" {
                let msg: BombExplode = Deserialize::deserialize(&mut de).unwrap();
                self.bomb_explode(msg);
            } else if msg_type == "destroy_item" {
                let msg: DestroyItem = Deserialize::deserialize(&mut de).unwrap();
                self.destroy_item(msg);
            } else if msg_type == "create_item" {
                let msg: CreateItem = Deserialize::deserialize(&mut de).unwrap();
                self.create_item(msg);
            } else if msg_type == "player_identity" {
                let msg: PlayerIdentity = Deserialize::deserialize(&mut de).unwrap();
                self.linked_id = Some(msg.id);
            } else if msg_type == "update_square" {
                let msg: UpdateSquare = Deserialize::deserialize(&mut de).unwrap();
                self.update_square(msg);
            } else if msg_type == "joined" {
                let msg: JoinedMsg = Deserialize::deserialize(&mut de).unwrap();
                if msg.success {
                    self.current_room_id = Some(msg.room);
                }
            } else {
                info!("unknown type: {}", msg_type);
            }
        }
    }

    pub fn process_rx(&mut self, buf: &mut Vec<u8>) {
        let mut pkts: Vec<Vec<u8>> = Vec::new();
        let rtp_buf = &mut self.rtp_buf;
        let size = buf.len() as u16;
        let mut parsed = 0;
        loop {
            let mut pkt_len = size - parsed;
            let mut store_remaining = true;
            let mut start = 0;

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
                    rtp_buf.data[rtp_buf.size as usize..((rtp_buf.size + eaten_bytes) as usize)]
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
                rtp_buf.data[rtp_buf.size as usize..((rtp_buf.size + stored_size) as usize)]
                    .copy_from_slice(&buf[(parsed as usize)..(parsed as usize + stored_size as usize)]);
                rtp_buf.size += stored_size;
                break;
            }

            let pkt = buf[(start as usize)..(start as usize + pkt_len as usize)].to_vec();
            buf.drain(..(start + pkt_len) as usize);
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