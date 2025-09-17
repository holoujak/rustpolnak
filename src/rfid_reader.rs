use bytes::BytesMut;
use futures::StreamExt;
use std::io;
use std::time::Duration;
use tokio::sync::broadcast::Sender;
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::Decoder;

type DevicePath = String;

pub struct LineReader;

impl Decoder for LineReader {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let newline = src.as_ref().iter().position(|b| *b == b'\n' || *b == b'\r');
        if let Some(n) = newline {
            let line = src.split_to(n + 1);
            return match std::str::from_utf8(line.as_ref()) {
                Ok(s) => {
                    let received = s.trim().to_string();
                    if !received.is_empty() {
                        Ok(Some(received))
                    } else {
                        Ok(None)
                    }
                }
                Err(_) => Err(io::Error::other("Invalid String")),
            };
        }
        Ok(None)
    }
}

#[derive(Clone, Debug)]
pub enum Event {
    Connected(DevicePath),
    Disconnected { device: DevicePath, error: String },
    Tag(String),
}

pub async fn rfid_serial(path: DevicePath, tx: Sender<Event>) {
    loop {
        match tokio_serial::new(&path, 115200).open_native_async() {
            Ok(mut port) => {
                port.set_exclusive(false).unwrap();
                let mut line_reader = LineReader.framed(port);
                tx.send(Event::Connected(path.to_string())).unwrap();

                loop {
                    match line_reader.next().await {
                        Some(Ok(line)) => {
                            tx.send(Event::Tag(line)).unwrap();
                        }
                        Some(Err(err)) => {
                            tx.send(Event::Disconnected {
                                device: path.to_string(),
                                error: err.to_string(),
                            })
                            .unwrap();
                        }
                        None => {
                            tx.send(Event::Disconnected {
                                device: path.to_string(),
                                error: "none received".to_string(),
                            })
                            .unwrap();
                            break;
                        }
                    }
                }
            }
            Err(err) => {
                tx.send(Event::Disconnected {
                    device: path.to_string(),
                    error: err.to_string(),
                })
                .unwrap();
            }
        };
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
