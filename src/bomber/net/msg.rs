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

use serde::{Deserialize, Serialize};
use super::super::gen::utils::Direction;
use super::super::gen::map::Map;

// This file contains messages which will be wrapped via msgpack.
// Each messages MUST have a unique msg_type.

/**
 * Generic message
 */
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Msg {
    pub msg_type: String, // TODO enum
}

impl Msg {
    pub fn new(msg_type: String) -> Msg {
        Msg {
            msg_type
        }
    }
}

/**
 * Message to send player details such as the name, its key, etc.
 */
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct PlayerMsg {
    pub msg_type: String,
    pub name: String,
}

impl PlayerMsg {
    pub fn new(name: String) -> PlayerMsg {
        PlayerMsg {
            name,
            msg_type: String::from("player")
        }
    }
}

/**
 * Message to join a room
 */
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct JoinMsg {
    pub msg_type: String,
    pub room: u64,
}

impl JoinMsg {
    pub fn new(room: u64) -> JoinMsg {
        JoinMsg {
            room,
            msg_type: String::from("join")
        }
    }
}

/**
 * Message to move a player
 */
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct MoveMsg {
    pub msg_type: String,
    pub direction: Direction,
}

impl MoveMsg {
    pub fn new(direction: Direction) -> MoveMsg {
        MoveMsg {
            msg_type: String::from("move"),
            direction,
        }
    }
}

/**
 * Message to move a player
 */
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct MapMsg {
    pub msg_type: String,
    pub map: Map,
}

impl MapMsg {
    pub fn new(map: Map) -> MapMsg {
        MapMsg {
            msg_type: String::from("map"),
            map,
        }
    }
}

/**
 * Message when a player join a room or lobby (room 0)
 */
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct JoinedMsg {
    pub msg_type: String,
    pub room: u64,
    pub success: bool
}

impl JoinedMsg {
    pub fn new(room: u64, success: bool) -> JoinedMsg {
        JoinedMsg {
            room,
            success,
            msg_type: String::from("joined")
        }
    }
}