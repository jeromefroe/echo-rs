extern crate mio;

use std::io::{Read, Write};
use std::time::Duration;
use std::collections::HashMap;
use mio::{Token, Ready, Events, Poll, PollOpt, Evented};
use mio::tcp::{TcpListener, TcpStream};
use mio::timer::Timer;

struct Connection {
    buf: [u8; 512],
    handle: TcpStream,
}

impl Connection {
    fn new(handle: TcpStream) -> Self {
        Connection {
            buf: [0; 512],
            handle: handle,
        }
    }
}

const SERVER: Token = Token(0);
const TIMER_TOKEN: Token = Token(1);

const ADDR: &'static str = "127.0.0.1:8888";

pub fn main() {
    let addr = ADDR.parse().unwrap();
    let server = TcpListener::bind(&addr).unwrap();
    let poll = Poll::new().unwrap();

    // create a buffer to read into
    let mut buf = [0; 512];

    let mut count = 1;
    let mut connections = HashMap::new();

    poll.register(&server, SERVER, Ready::readable(), PollOpt::edge()).unwrap();

    // Create storage for events
    let mut events = Events::with_capacity(1024);

    let mut timer = Timer::default();
    poll.register(&timer, TIMER_TOKEN, Ready::readable(), PollOpt::edge())
        .unwrap();
    let flush_timeout = Duration::from_millis(5000);
    timer.set_timeout(flush_timeout, String::from("hello from 1s ago")).ok();

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

                    let conn = Connection::new(stream);

                    connections.insert(count, conn);

                }
                TIMER_TOKEN => {
                    println!("number of open connections: {}", connections.len());
                    timer.set_timeout(flush_timeout, String::from("hello from 1s ago")).ok();
                }
                Token(c) => {
                    let n;

                    // need to get stream in a seperate scope in case client closes the connection
                    // in which case we want to drop it
                    {
                        let conn = connections.get_mut(&c).unwrap();

                        n = match conn.handle.read(&mut buf) {
                            Ok(m) => m,
                            Err(e) => panic!("got an error when reading from connection: {}", e),
                        };

                        if n != 0 {
                            println!("read {} bytes from client", n);

                            match conn.handle.write(&buf[..n]) {
                                Ok(n) => println!("wrote {} bytes from client", n),
                                Err(e) => panic!("got an error when writing to connection: {}", e),
                            }
                        }
                    }

                    if n == 0 {
                        // EOF, client closed connection so we can drop it now
                        println!("client closed connection");
                        let conn = connections.remove(&c).unwrap();
                        poll.deregister(&conn.handle);
                    }
                }
            }
        }
    }
}