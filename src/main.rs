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

use bomber::core::{Client, KeyHandler};
use bomber::net::{TlsClient, TlsClientConfig};

use std::sync::{Arc, Mutex};
use std::thread;
use futures::sync::mpsc;

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
}