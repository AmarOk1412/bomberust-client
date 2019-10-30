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

use futures::Stream;
use std::fmt;
use std::fs;
use std::io::BufReader;
use std::net::SocketAddr;
use std::sync::{ Arc, Mutex };
use tokio::io::{ AsyncRead, AsyncWrite };
use tokio::net::TcpStream;
use tokio::prelude::{ Async, Future };
use tokio::timer::Interval;
use tokio_rustls::{ TlsConnector, rustls::ClientConfig };

use super::super::core::Client;

pub enum ConnectionState {
    Connecting,
    Connected,
    Disconnected,
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConnectionState::Connecting => write!(f, "Connecting"),
            ConnectionState::Connected => write!(f, "Connected"),
            ConnectionState::Disconnected => write!(f, "Disconnected"),
        }
    }
}

pub struct TlsClientConfig {
    pub server_state: Arc<Mutex<Option<ConnectionState>>>,
    pub addr: String,
    pub cert: String,
    pub client: Arc<Mutex<Client>>
}

pub struct TlsClient {
}

use futures::future;

impl TlsClient {
    pub fn start(client_config: &TlsClientConfig) {

        let mut cafile: Option<&str> = None;
        if !client_config.cert.is_empty() {
            cafile = Some(&*client_config.cert);
        }

        let mut config = ClientConfig::new();
        if let Some(cafile) = cafile {
            let mut pem = BufReader::new(fs::File::open(cafile).unwrap());
            config.root_store.add_pem_file(&mut pem).unwrap();
        } else {
            config.root_store.add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
        }
        let config = TlsConnector::from(Arc::new(config));

        let client = client_config.client.clone();

        let server_state = client_config.server_state.clone();
        let server_state_err = client_config.server_state.clone();
        let server_state_err2 = client_config.server_state.clone();
        let server: SocketAddr = client_config.addr
            .parse()
            .expect("Unable to parse socket address. Please verify config.json");

        let socket = TcpStream::connect(&server);
        let done = socket
        .and_then(move |stream| {
            let domain = webpki::DNSNameRef::try_from_ascii_str("localhost").unwrap();
            config.connect(domain, stream)
        })
        .and_then(|socket| {
            // TODO Framed buffer for RTP packets?
            let (mut rx, mut tx) = socket.split();
            let connected = Arc::new(Mutex::new(true));
            let connected_cln = connected.clone();
            let worker = Interval::new_interval(std::time::Duration::from_millis(1))
            .take_while(move |_| {
                future::ok(*connected.lock().unwrap())
            })
            .for_each(move |_| {
               *server_state.lock().unwrap() = Some(ConnectionState::Connected);
                if client.lock().unwrap().send_buf.lock().unwrap().is_some() {
                    *connected_cln.lock().unwrap() = tx.poll_write(
                        &*client.lock().unwrap().send_buf.lock().as_ref().unwrap().as_ref().unwrap()
                    ).is_ok();
                    *client.lock().unwrap().send_buf.lock().unwrap() = None;
                }

                if !*connected_cln.lock().unwrap() {
                    return Ok(());
                }

                let mut buffer = vec![0u8; 65536];
                match rx.poll_read(&mut buffer) {
                    Ok(Async::Ready(n)) => {
                        if n > 0 {
                            client.lock().unwrap().process_rx(&mut buffer[..n].to_vec());
                        } else {
                            *server_state.lock().unwrap() = Some(ConnectionState::Disconnected);
                            *connected_cln.lock().unwrap() = false;
                        }
                    }
                    Ok(Async::NotReady) => {}
                    _ => {
                        *server_state.lock().unwrap() = Some(ConnectionState::Disconnected);
                        *connected_cln.lock().unwrap() = false;
                    }
                };
                return Ok(());
            }).map_err(move |_e| {
               *server_state_err.lock().unwrap() = Some(ConnectionState::Disconnected);
            });
            tokio::spawn(worker);
            return Ok(());
        }).map_err(move |_e| {
            *server_state_err2.lock().unwrap() = Some(ConnectionState::Disconnected);
        });

        tokio::run(done);
    }
}