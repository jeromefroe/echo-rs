extern crate mio;

use std::io::{Read, Write};
use std::collections::HashMap;
use mio::{Token, Ready, Events, Poll, PollOpt};
use mio::tcp::TcpListener;

const SERVER: Token = Token(0);
const ADDR: &'static str = "127.0.0.1:8888";

pub fn main() {
    let addr = ADDR.parse().unwrap();
    let server = TcpListener::bind(&addr).unwrap();
    let poll = Poll::new().unwrap();

    // create a buffer to read into
    let mut buf = [0; 512];

    let mut count = 0;
    let mut connections = HashMap::new();

    poll.register(&server, SERVER, Ready::readable(), PollOpt::edge()).unwrap();

    // Create storage for events
    let mut events = Events::with_capacity(1024);

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match event.token() {
                SERVER => {
                    let (stream, client_addr) = match server.accept() {
                        Ok((stream, client_addr)) => (stream, client_addr),
                        Err(e) => panic!("got an error when accepting a connection: {}", e),
                    };

                    println!("connection from: {}", client_addr);
                    count += 1;

                    poll.register(&stream, Token(count), Ready::readable(), PollOpt::edge())
                        .unwrap();

                    connections.insert(count, Box::new(stream));

                }
                Token(c) => {
                    let n;

                    // need to get stream in a seperate scope in case client closes the connection
                    // in which case we want to drop it
                    {
                        let stream = connections.get_mut(&c).unwrap();

                        n = match stream.read(&mut buf) {
                            Ok(m) => m,
                            Err(e) => panic!("got an error when reading from connection: {}", e),
                        };

                        if n != 0 {
                            println!("read {} bytes from client", n);

                            match stream.write(&buf[..n]) {
                                Ok(n) => println!("wrote {} bytes from client", n),
                                Err(e) => panic!("got an error when writing to connection: {}", e),
                            }
                        }
                    }

                    if n == 0 {
                        // EOF, client closed connection so we can drop it now
                        println!("client closed connection");
                        connections.remove(&c);
                    }
                }
            }
        }
    }
}