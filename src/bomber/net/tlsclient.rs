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


use std::fs;
use std::io::BufReader;
use std::net::ToSocketAddrs;
use tokio::net::TcpStream;
use tokio::prelude::Future;
use tokio_rustls::{ TlsConnector, rustls::ClientConfig };
use std::sync::{Arc, Mutex};
use std::{thread, time};

use super::super::core::Client;

pub struct TlsClientConfig {
    pub host: String,
    pub port: u16,
    pub cert: String,
    pub client: Arc<Mutex<Client>>
}

pub struct TlsClient {
}

impl TlsClient {
    pub fn start(client_config: &TlsClientConfig) {

        let addr = (&*client_config.host, client_config.port).to_socket_addrs().unwrap().next().unwrap();
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

        let socket = TcpStream::connect(&addr);

        let client = client_config.client.clone();

        let done = socket
            .and_then(move |stream| {
                let domain = webpki::DNSNameRef::try_from_ascii_str("localhost").unwrap();
                config.connect(domain, stream)
            })
            .and_then(move |stream| {
                client.lock().unwrap().set_stream(stream);
                let process_delay = time::Duration::from_nanos(100);
                loop {
                    if !client.lock().unwrap().process_stream() {
                        break;
                    }
                    thread::sleep(process_delay);
                }
                Ok(())
            })
            .map(drop)
            .map_err(|err| eprintln!("{:?}", err));

        tokio::run(done);
    }
}