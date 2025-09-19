use bytes::{Buf, BytesMut};
use futures::StreamExt;
use log::trace;
use nom::{bytes::complete::take, multi::count, number::complete::be_u8, IResult, Parser};
use std::time::Duration;
use tokio::sync::broadcast::Sender;
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::Decoder;

type DevicePath = String;

const HEADER_MAGIC: [u8; 2] = [0x43, 0x54];
const LEN_BYTES: usize = 2;
/// header size including 2 magic bytes and 2 byte length
const HEADER_SIZE: usize = HEADER_MAGIC.len() + LEN_BYTES;

#[derive(Debug, PartialEq)]
pub struct Frame {
    cmd: Cmd,
    addr: u8,
    status: Status,
}

#[derive(Debug, PartialEq)]
pub struct Tag {
    tag_type: u8,
    ant2: u8,
    rssi: u8,
    tag_id: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub enum Cmd {
    ActiveData { dev_sn: [u8; 7], tags: Vec<Tag> },
    UnknownCommand(u8),
}

#[derive(Debug, PartialEq)]
pub enum Status {
    Fail,
    Success,
}

#[derive(Clone, Debug)]
pub enum Event {
    Connected(DevicePath),
    Disconnected { device: DevicePath, error: Error },
    Tag(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    InvalidHeader,
    WrongChecksum { received: u8, expected: u8 },
    Eof,
    IOError(String),
    ParseError,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IOError(value.to_string())
    }
}

fn checksum(data: &[u8]) -> u8 {
    let sum = data.iter().fold(0u8, |acc, &x| acc.wrapping_add(x));
    (!sum).wrapping_add(1)
}

fn parse_tag(input: &[u8]) -> IResult<&[u8], Tag> {
    let (input, tag_length) = be_u8(input)?;
    let (input, tag_type) = be_u8(input)?;
    let (input, ant2) = be_u8(input)?;
    let (input, tag_id) = take(tag_length - 3)(input)?;
    let (input, rssi) = be_u8(input)?;

    Ok((
        input,
        Tag {
            ant2,
            tag_type,
            rssi,
            tag_id: Vec::from(tag_id),
        },
    ))
}

fn parse_active_data(input: &[u8]) -> IResult<&[u8], Cmd> {
    let (input, dev_sn) = take(7usize)(input)?;
    let (input, tag_number) = be_u8(input)?;
    let (input, tags) = count(parse_tag, tag_number as usize).parse(input)?;

    Ok((
        input,
        Cmd::ActiveData {
            dev_sn: {
                let mut dev_sn_buf = [0u8; 7];
                dev_sn_buf.copy_from_slice(dev_sn);
                dev_sn_buf
            },
            tags,
        },
    ))
}

fn parse(input: &[u8]) -> IResult<&[u8], Frame> {
    let (input, addr) = be_u8(input)?;
    let (input, cmd) = be_u8(input)?;
    let (input, status) = be_u8(input)?;

    let (input, cmd) = match cmd {
        0x45 => parse_active_data(input)?,
        _ => (input, Cmd::UnknownCommand(cmd)),
    };

    Ok((
        input,
        Frame {
            cmd,
            addr,
            status: match status {
                1 => Status::Success,
                _ => Status::Fail,
            },
        },
    ))
}

pub struct RFIDProtocol;

impl RFIDProtocol {
    /// Get the CRC valid frame without header, length and checksum
    fn payload_with_valid_crc<'a>(&self, frame: &'a [u8]) -> Result<&'a [u8], Error> {
        let (received_crc, frame_without_checksum) = frame.split_last().unwrap();
        let calculated_crc = checksum(frame_without_checksum);
        if *received_crc == calculated_crc {
            Ok(&frame_without_checksum[HEADER_SIZE..])
        } else {
            Err(Error::WrongChecksum {
                received: *received_crc,
                expected: calculated_crc,
            })
        }
    }

    fn parse_frame(&mut self, frame: &[u8]) -> Result<Frame, Error> {
        match parse(self.payload_with_valid_crc(frame)?) {
            Ok((_, frame)) => Ok(frame),
            Err(_err) => Err(Error::ParseError),
        }
    }
}

impl Decoder for RFIDProtocol {
    type Item = Frame;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // synchronize with header bytes
        for (index, expected_byte) in HEADER_MAGIC.iter().enumerate() {
            match src.get(index) {
                Some(received_byte) => {
                    if received_byte != expected_byte {
                        // skip until we find the header bytes
                        src.advance(index + 1);
                        return Err(Error::InvalidHeader);
                    }
                }
                // header is not complete
                None => return Ok(None),
            }
        }

        // get the 2 byte size
        let len = match src
            .get(HEADER_MAGIC.len()..HEADER_SIZE)
            .and_then(|b| <[u8; 2]>::try_from(b).ok())
        {
            Some(bytes) => u16::from_be_bytes(bytes) as usize,
            None => return Ok(None),
        };

        match src.get(..HEADER_SIZE + len) {
            Some(frame) => {
                let resp = match self.parse_frame(frame) {
                    Ok(frame) => {
                        trace!("Received: {frame:?}");
                        Ok(Some(frame))
                    }
                    Err(err) => Err(err),
                };
                // remove frame from the rx buffer
                src.advance(len + HEADER_SIZE);
                resp
            }
            None => Ok(None),
        }
    }
}

pub async fn rfid_serial(path: DevicePath, tx: Sender<Event>) {
    loop {
        match tokio_serial::new(&path, 115200).open_native_async() {
            Ok(mut port) => {
                port.set_exclusive(false).unwrap();
                let mut frame_reader = RFIDProtocol.framed(port);
                tx.send(Event::Connected(path.to_string())).unwrap();

                loop {
                    match frame_reader.next().await {
                        Some(Ok(frame)) => match frame.cmd {
                            Cmd::ActiveData { tags, .. } => {
                                for tag in tags {
                                    let tag: String = tag
                                        .tag_id
                                        .into_iter()
                                        .map(|b| format!("{b:02X}"))
                                        .collect();
                                    tx.send(Event::Tag(tag.to_string())).unwrap();
                                }
                            }
                            Cmd::UnknownCommand(cmd) => {
                                println!("Unknown command: {}", cmd);
                            }
                        },
                        Some(Err(err)) => {
                            tx.send(Event::Disconnected {
                                device: path.to_string(),
                                error: err,
                            })
                            .unwrap();
                        }
                        None => {
                            tx.send(Event::Disconnected {
                                device: path.to_string(),
                                error: Error::Eof,
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
                    error: Error::IOError(err.to_string()),
                })
                .unwrap();
            }
        };
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_data() {
        let mut reader = RFIDProtocol {};
        let mut buf = BytesMut::new();
        let res = reader.decode(&mut buf);
        assert!(res.unwrap().is_none());
        assert!(buf.is_empty());
    }

    #[test]
    fn one_byte_good_start() {
        let mut reader = RFIDProtocol {};
        let mut buf = BytesMut::from(&[0x43][..]);
        let res = reader.decode(&mut buf);
        assert!(res.unwrap().is_none());
        assert_eq!(buf.as_ref(), &[0x43]);
    }

    #[test]
    fn one_byte_wrong_start() {
        let mut reader = RFIDProtocol {};
        let mut buf = BytesMut::from(&[0x11][..]);
        let res = reader.decode(&mut buf);
        assert_eq!(res.unwrap_err(), Error::InvalidHeader);
        assert!(buf.is_empty());
    }

    #[test]
    fn two_bytes_good() {
        let mut reader = RFIDProtocol {};
        let mut buf = BytesMut::from(&[0x43, 0x54][..]);
        let res = reader.decode(&mut buf);
        assert!(res.unwrap().is_none());
        assert_eq!(buf.as_ref(), &[0x43, 0x54]);
    }

    #[test]
    fn two_bytes_bad() {
        let mut reader = RFIDProtocol {};
        let mut buf = BytesMut::from(&[0x43, 0xaa][..]);
        let res = reader.decode(&mut buf);
        assert_eq!(res.unwrap_err(), Error::InvalidHeader);
        assert!(buf.is_empty());
    }

    #[test]
    fn incomplete_header() {
        let mut reader = RFIDProtocol {};
        let mut buf = BytesMut::from(
            &[
                0x43, 0x54, // header
                0x00, // length - just one byte
            ][..],
        );
        let res = reader.decode(&mut buf);
        assert!(res.unwrap().is_none());
        assert_eq!(buf.as_ref(), &[0x43, 0x54, 0x00]);
    }

    #[test]
    fn incomplete_payload_header() {
        let mut reader = RFIDProtocol {};
        let mut buf = BytesMut::from(
            &[
                0x43, 0x54, // header
                0x00, 0x02, // length
                0x01, // part of payload
            ][..],
        );
        let res = reader.decode(&mut buf);
        assert!(res.unwrap().is_none());
        assert_eq!(buf.as_ref(), &[0x43, 0x54, 0x00, 0x02, 0x01]);
    }

    #[test]
    fn wrong_crc() {
        let mut reader = RFIDProtocol {};
        let mut buf = BytesMut::from(
            &[
                0x43, 0x54, // header
                0x00, 0x1c, // length
                0x00, // address
                0x45, // command
                0x01, // status
                0xc3, 0x85, 0x14, 0x01, 0x02, 0x01, 0xa4, 0x01, 0x0f, 0x01, 0x01, 0x85, 0x13, 0x33,
                0x6d, 0xb6, 0x2d, 0x6d, 0xc2, 0xd4, 0x20, 0x6c, 0xe7, 0xbc, // payload
                0x11, // wrong crc
            ][..],
        );
        let res = reader.decode(&mut buf);
        assert_eq!(
            res.unwrap_err(),
            Error::WrongChecksum {
                received: 0x11,
                expected: 0xa4
            }
        );
        assert!(buf.is_empty());
    }

    #[test]
    fn unknown_command() {
        let mut reader = RFIDProtocol {};
        let mut buf = BytesMut::from(
            &[
                0x43, 0x54, // header
                0x00, 0x04, // length
                0x00, // address
                0x42, // command
                0x01, // status
                0x22, // crc
            ][..],
        );
        let res = reader.decode(&mut buf);
        assert_eq!(
            res.unwrap().unwrap(),
            Frame {
                cmd: Cmd::UnknownCommand(0x42),
                status: Status::Success,
                addr: 0,
            }
        );
        assert!(buf.is_empty());
    }

    #[test]
    fn leftover() {
        let mut reader = RFIDProtocol {};
        let mut buf = BytesMut::from(
            &[
                0x43, 0x54, // header
                0x00, 0x04, // length
                0x00, // address
                0x42, // command
                0x00, // status
                0x23, // crc
                0xAA, 0xBB, // another possible frame
            ][..],
        );
        let res = reader.decode(&mut buf);
        assert_eq!(
            res.unwrap().unwrap(),
            Frame {
                cmd: Cmd::UnknownCommand(0x42),
                status: Status::Fail,
                addr: 0,
            }
        );
        assert_eq!(buf.as_ref(), [0xAA, 0xBB]);
    }

    #[test]
    fn active_data_one_tag() {
        let mut reader = RFIDProtocol {};
        let mut buf = BytesMut::from(
            &[
                0x43, 0x54, // header
                0x00, 0x1c, // length
                0x00, // address
                0x45, // command
                0x01, // status
                0xc3, 0x85, 0x14, 0x01, 0x02, 0x01, 0xa4, // dev_sn
                0x01, // number of tags
                0x0f, // size of tag
                0x01, // tag_type
                0x01, // ant2
                0x85, 0x13, 0x33, 0x6d, 0xb6, 0x2d, 0x6d, 0xc2, 0xd4, 0x20, 0x6c, 0xe7, // tag
                0xbc, // rssi
                0xa4, // crc
            ][..],
        );
        let res = reader.decode(&mut buf);
        assert_eq!(
            res.unwrap().unwrap(),
            Frame {
                addr: 0,
                status: Status::Success,
                cmd: Cmd::ActiveData {
                    dev_sn: [0xc3, 0x85, 0x14, 0x01, 0x02, 0x01, 0xa4],
                    tags: vec![Tag {
                        tag_type: 1,
                        ant2: 1,
                        rssi: 0xbc,
                        tag_id: vec![
                            0x85, 0x13, 0x33, 0x6d, 0xb6, 0x2d, 0x6d, 0xc2, 0xd4, 0x20, 0x6c, 0xe7,
                        ]
                    }]
                }
            }
        );
        assert!(buf.is_empty());
    }

    #[test]
    fn active_data_corrupted() {
        let mut reader = RFIDProtocol {};
        let mut buf = BytesMut::from(
            &[
                0x43, 0x54, // header
                0x00, 0x12, // length
                0x00, // address
                0x45, // command
                0x01, // status
                0xc3, 0x85, 0x14, 0x01, 0x02, 0x01, 0xa4, // dev_sn
                0x01, // number of tags
                0x0f, // size of tag
                0x01, // tag_type
                0x01, // ant2
                0x85, 0x13, // just one byte in tag
                0xbc, // rssi
                0xa7, // crc
            ][..],
        );
        let res = reader.decode(&mut buf);
        assert_eq!(res.unwrap_err(), Error::ParseError);
        assert!(buf.is_empty());
    }
}
