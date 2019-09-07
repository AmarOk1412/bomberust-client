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

use futures::Async;
use tokio_rustls::client::TlsStream;
use tokio::net::TcpStream;
use tokio::io::AsyncRead;
use std::io::Write;
use std::sync::{Arc, Mutex};

pub struct RtpBuf {
    data: [u8; 65536],
    size: u16,
    wanted: u16,
}

pub struct Socket {
    stream: Option<TlsStream<TcpStream>>,
    rtp_buf: RtpBuf,
}

pub struct Client
{
    pub socket: Socket,
    pub send_buf: Arc<Mutex<Option<Vec<u8>>>>,
}

impl Client {
    pub fn new(send_buf: Arc<Mutex<Option<Vec<u8>>>>) -> Client {
        Client {
            send_buf: send_buf,
            socket: Socket {
                stream: None,
                rtp_buf: RtpBuf {
                    data: [0; 65536],
                    size: 0,
                    wanted: 0,
                } 
            }
        }
    }

    pub fn set_stream(&mut self, stream: TlsStream<TcpStream>) {
        self.socket.stream = Some(stream);
    }

    pub fn parse_rtp(&mut self, pkt: String) {
        debug!("rx:{}", pkt);
    }

    pub fn process_stream(&mut self) -> bool {
        if self.socket.stream.is_none() {
            return true;
        }
        let mut buf = [0; 1024];
        let mut result = true;
        let mut pkts: Vec<String> = Vec::new();
        let rtp_buf = &mut self.socket.rtp_buf;
        match self.socket.stream.as_mut().unwrap().poll_read(&mut buf) {
            Ok(Async::Ready(n)) => {
                result = n != 0;
                let size = n as u16;
                if result {
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
                        pkts.push(String::from_utf8(pkt).unwrap_or(String::new()));

                        if parsed >= size {
                            break;
                        }
                    }
                }
            },
            Ok(Async::NotReady) => {}
            Err(_) => { result = false; }
        };
        for pkt in pkts {
            self.parse_rtp(pkt);
        }
        if !result {
            return false;
        }

        if self.send_buf.lock().unwrap().is_some() {
            match self.socket.stream.as_mut().unwrap().write(
                &*self.send_buf.lock().as_ref().unwrap().as_ref().unwrap()) {
                Err(_) => {
                    result = false;
                }
                _ => {}
            }
            *self.send_buf.lock().unwrap() = None;
        }
    
        result
    }
}