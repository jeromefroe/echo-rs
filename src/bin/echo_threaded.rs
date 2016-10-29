use std::thread;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

fn handle_client(mut stream: TcpStream) {
    let mut buf = [0; 512];

    loop {
        let n = match stream.read(&mut buf) {
            Ok(n) => {
                if n == 0 {
                    // EOF
                    println!("client closed connection");
                    break;
                }
                n
            }
            Err(e) => panic!("got an error reading from a connection: {}", e),
        };

        println!("read {} bytes from the client", n);

        match stream.write(&buf[..n]) {
            Ok(n) => println!("wrote {} bytes to the client", n),
            Err(e) => panic!("got an error writing to connection: {}", e),
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8888").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_client(stream));
            }
            Err(e) => panic!("got an error accepting connection: {}", e),
        }
    }
}