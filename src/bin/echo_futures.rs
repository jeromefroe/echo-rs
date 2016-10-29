extern crate futures;
extern crate tokio_core;

use std::net::SocketAddr;

use futures::Future;
use futures::stream::Stream;
use tokio_core::io::{copy, Io};
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;

fn main() {
    let addr = "127.0.0.1:8080".to_string();
    let addr = addr.parse::<SocketAddr>().unwrap();

    let mut l = Core::new().unwrap();
    let handle = l.handle();

    let socket = TcpListener::bind(&addr, &handle).unwrap();
    println!("Listening on: {}", addr);

    let done = socket.incoming().for_each(move |(socket, addr)| {
        let pair = futures::lazy(|| futures::finished(socket.split()));
        let amt = pair.and_then(|(reader, writer)| copy(reader, writer));

        let msg = amt.map(move |amt| println!("wrote {} bytes to {}", amt, addr))
            .map_err(|e| {
                panic!("error: {}", e);
            });
        handle.spawn(msg);

        Ok(())
    });
    l.run(done).unwrap();
}