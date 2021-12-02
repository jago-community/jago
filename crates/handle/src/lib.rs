use context::Context;

use std::iter::Peekable;

pub fn grasp<'a>(
    input: &mut Peekable<impl Iterator<Item = String>>,
    _: &'a mut Context,
) -> Result<(), Error> {
    match input.peek() {
        Some(name) if name == "handle" => {
            drop(input.next());

            if input.peek().is_some() {
                ping(input)
            } else {
                watch(input)
            }
        }
        _ => Ok(()),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
    #[error("Packet {0}")]
    Packet(#[from] bincode::Error),
    #[error("Address {0}")]
    Address(#[from] std::net::AddrParseError),
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
enum Message {
    Arguments(Vec<String>),
}

use std::net::UdpSocket;

pub fn watch<'a>(_input: &mut Peekable<impl Iterator<Item = String>>) -> Result<(), Error> {
    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async move {
        let sock = UdpSocket::bind("0.0.0.0:1342")?;

        loop {
            let mut size = [0u8; 1];

            let _ = sock.recv_from(&mut size)?;

            let mut buffer = vec![0u8; size[0] as usize];

            let _ = sock.recv_from(&mut buffer)?;

            let message = bincode::deserialize::<Message>(&buffer)?;

            handle_packet(message)
        }
    })
}

fn handle_packet(packet: Message) {
    match packet {
        Message::Arguments(arguments) => {
            log::info!("Arguments: {:?}", arguments);
        }
    }
}

use std::{net::SocketAddrV4, str::FromStr};

pub fn ping(arguments: impl Iterator<Item = String>) -> Result<(), Error> {
    let self_address = SocketAddrV4::from_str("0.0.0.0:2134")?;
    let host_address = SocketAddrV4::from_str("0.0.0.0:1342")?;

    let sock = UdpSocket::bind(self_address)?;

    let message = bincode::serialize(&Message::Arguments(arguments.collect()))?;

    sock.send_to(&[message.len() as u8], host_address)?;
    sock.send_to(&message, host_address)?;

    Ok(())
}
