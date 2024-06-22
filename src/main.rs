use std::{io::{Read, Write}, net::{TcpListener, TcpStream}};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("New connection from {:?}", stream.peer_addr());

                tokio::spawn(async move { handle_connection(&mut stream).await });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

async fn handle_connection(stream: &mut TcpStream) {
    let mut buffer = [0; 256];

    stream.read(&mut buffer).unwrap();

    match std::str::from_utf8(&buffer).unwrap() {
        message if message.starts_with("GET / ") => {
            stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
        },
        message if message.starts_with("GET /echo/") => handle_echo(stream, message).await,
        message if message.starts_with("GET /user-agent") => handle_user_agent(stream, message).await,
        _ => {
            stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
        },
    }
}

async fn handle_echo(stream: &mut TcpStream, message: &str) {
    let path = message.split(' ').nth(1).unwrap();

    let parameter = path.split('/').nth(2).unwrap();

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{parameter}",
        parameter.len()
    );

    stream.write_all(response.as_bytes()).unwrap();
}

async fn handle_user_agent(stream: &mut TcpStream, message: &str) {
    let mut user_agent = String::new();

    for line in message.lines() {
        if line.starts_with("User-Agent") {
            user_agent = line.split(" ")
                .skip(1)
                .collect::<String>();
        }
    }

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{user_agent}",
        user_agent.len(),
    );

    stream.write_all(response.as_bytes()).unwrap();
}
