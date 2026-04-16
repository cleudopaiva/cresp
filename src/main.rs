use std::io::ErrorKind;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::{Arc, RwLock};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    spawn,
};

use cresp::{command::Command, frame::parse_frame, store::Store};

#[tokio::main]
async fn main() {
    let store = Arc::new(RwLock::new(Store::new()));
    let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 6379);
    let listener_result = TcpListener::bind(socket).await;

    let listener = match listener_result {
        Ok(listener) => listener,
        Err(error) => match error.kind() {
            ErrorKind::AddrInUse => panic!("This address is already in use {}", socket),
            _ => {
                panic!("Error opening tcp listener: {error:?}");
            }
        },
    };

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let store_clone = Arc::clone(&store);

        spawn(async move {
            handle_connection(stream, store_clone).await;
        });
    }
}

async fn handle_connection(mut stream: TcpStream, store: Arc<RwLock<Store>>) {
    let (reader, mut writer) = stream.split();
    let mut buf_reader = BufReader::new(reader);
    loop {
        let buffer = match buf_reader.fill_buf().await {
            Ok(buffer) => buffer,
            Err(e) => {
                eprintln!("Error reading sockert: {e}");
                break;
            }
        };

        if buffer.is_empty() {
            break;
        }

        let input = match std::str::from_utf8(buffer) {
            Ok(input) => input,
            Err(_) => {
                if let Err(e) = writer
                    .write_all(b"-ERR Protocol Error: Invalid UTF-8\r\n")
                    .await
                {
                    eprintln!("Error writing response: {e}");
                }

                break;
            }
        };
        match parse_frame(input) {
            Ok((remainder, frame)) => {
                let consumed = buffer.len() - remainder.len();
                buf_reader.consume(consumed);

                match Command::try_from(frame) {
                    Ok(cmd) => {
                        if let Err(e) = cmd.execute(&store, &mut writer).await {
                            eprintln!("Error executing command: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        let err_message = format!("-ERR {}\r\n", e);
                        if let Err(e) = writer.write_all(err_message.as_bytes()).await {
                            eprintln!("Error writing response: {e}");
                            break;
                        }
                    }
                }
            }
            Err(nom::Err::Incomplete(_)) => {
                continue;
            }
            Err(e) => {
                let err_msg = format!("-ERR Protocol Error: Invalid RESP format {e:?}\r\n");
                if let Err(e) = writer.write_all(err_msg.as_bytes()).await {
                    eprintln!("Error writing response: {e}");

                    break;
                }

                let bytes_to_consume = buffer
                    .iter()
                    .position(|&b| b == b'\n')
                    .map(|pos| pos + 1)
                    .unwrap_or(buffer.len());
                buf_reader.consume(bytes_to_consume);
            }
        };
    }
}
