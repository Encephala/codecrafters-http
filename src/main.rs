use std::{io::Read, io::Write, net::TcpListener};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 64];

                stream.read(&mut buffer).unwrap();

                match std::str::from_utf8(&buffer).unwrap() {
                    message if message.starts_with("GET / ") => {
                        stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
                    },
                    message if message.starts_with("GET /echo/") => {
                        let path = message.split(' ').nth(1).unwrap();

                        let parameter = path.split('/').nth(2).unwrap();

                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{parameter}",
                            parameter.len()
                        );

                        stream.write_all(response.as_bytes()).unwrap();
                    },
                    _ => {
                        stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
                    },
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
