use std::{io::{Read, Write}, net::{TcpListener, TcpStream}, path::PathBuf};

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
        message if message.starts_with("GET /echo/") => handle_echo(stream, message),
        message if message.starts_with("GET /user-agent") => handle_user_agent(stream, message),

        // This is great logic, don't @me
        message if message.contains("/files/") => handle_file(stream, message),
        _ => {
            stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
        },
    }
}

fn handle_echo(stream: &mut TcpStream, message: &str) {
    let path = message.split(' ').nth(1).unwrap();

    let parameter = path.split('/').nth(2).unwrap();

    let mut encoding = None;
    
    for line in message.lines() {
        if let Some(encodings) = line.strip_prefix("Accept-Encoding: ") {
            let accepted_encodings = encodings.split(", ").collect::<Vec<_>>();

            encoding = Some(accepted_encodings);
        }
    }

    let response = match encoding {
        Some(encodings) => {
            if encodings.contains(&"gzip") {
                let mut gzip_process = std::process::Command::new("gzip")
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .spawn()
                    .expect("failed to encode");

                gzip_process.stdin.as_mut().unwrap().write_all(parameter.as_bytes()).unwrap();

                let encoded = gzip_process.wait_with_output().unwrap().stdout;

                let message = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\n\r\n",
                    encoded.len()
                );

                message.bytes().chain(encoded).collect::<Vec<_>>()
            } else {
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{parameter}",
                    parameter.len()
                ).into_bytes()
            }
        },
        None => {
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{parameter}",
                parameter.len()
            ).into_bytes()
        },
    };

    stream.write_all(&response).unwrap();
}

fn handle_user_agent(stream: &mut TcpStream, message: &str) {
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

fn handle_file(stream: &mut TcpStream, message: &str) {
    let path = message.split(' ').nth(1).unwrap();

    let parameter = path.split('/').nth(2).unwrap();

    let args: Vec<String> = std::env::args().collect();

    let mut directory = String::new();
    let mut next_is_path = false;

    for arg in args {
        if arg == "--directory" {
            next_is_path = true;
            continue;
        }

        if next_is_path {
            directory = arg;
            break;
        }
    }

    let file_path = PathBuf::from(directory).join(parameter);

    if message.starts_with("GET") {
        if !file_path.exists() {
            let message = "HTTP/1.1 404 Not Found\r\n\r\n";

            stream.write_all(message.as_bytes()).unwrap();
        }

        let contents = std::fs::read(file_path).unwrap();

        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
            contents.len()
        );

        let message = header.bytes().chain(contents).collect::<Vec<_>>();

        stream.write_all(&message).unwrap();

        return;
    }

    if message.starts_with("POST") {
        let mut body = message.split("\r\n\r\n")
            .nth(1)
            .unwrap()
            .to_owned();

        // I initialise buffer as a bunch of nulls, so have to remove them from the end
        // There are more efficient ways to do this, but HTML just sends strings,
        // and strings are null terminated, the body can't contain a null so it works 🤷
        body.retain(|character| character as u8 != 0);

        std::fs::write(file_path, body).unwrap();

        stream.write_all(b"HTTP/1.1 201 Created\r\n\r\n").unwrap();
    }
}
