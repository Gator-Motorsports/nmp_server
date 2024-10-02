use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use clap::Parser;
use codec::MessageCodec;
use futures_util::SinkExt;
use message::{Data, Message};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

mod codec;
mod connection_handler;
mod message;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    program_name: Option<String>,
    #[arg(short, long)]
    tcp_addr: SocketAddr,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let connection = TcpStream::connect(args.tcp_addr).await.unwrap();
    let mut framed = Framed::new(connection, MessageCodec::new());

    let mut started = false;

    if let Some(program_name) = args.program_name {
        match program_name.as_str() {
            "100hz" => loop {
                if !started {
                    println!("Starting test: {}", program_name);
                }
                started = true;
                framed
                    .send(Message::Signal("100hz".into(), Data::Integer(10)))
                    .await
                    .unwrap();
                tokio::time::sleep(Duration::from_millis(10)).await;
            },
            "1khz" => loop {
                if !started {
                    println!("Starting test: {}", program_name);
                }
                started = true;
                framed
                    .send(Message::Signal("1khz".into(), Data::Integer(10)))
                    .await
                    .unwrap();
                tokio::time::sleep(Duration::from_millis(1)).await;
            },
            "1khz_o" => {
                if !started {
                    println!("Starting test: {}", program_name);
                }
                started = true;
                let start = Instant::now();
                loop {
                    framed
                        .send(Message::Signal(
                            "1khz_o".into(),
                            Data::Float(
                                100.0 * (Instant::elapsed(&start).as_micros() as f64).sin(),
                            ),
                        ))
                        .await
                        .unwrap();
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            }
            "1khz_4" => loop {
                println!("Starting test: {}", program_name);
                framed
                    .send(Message::Signal("1khz_a".into(), Data::Integer(10)))
                    .await
                    .unwrap();
                framed
                    .send(Message::Signal("1khz_b".into(), Data::Float(-12.0)))
                    .await
                    .unwrap();
                framed
                    .send(Message::Signal("1khz_c".into(), Data::Bool(true)))
                    .await
                    .unwrap();
                framed
                    .send(Message::Signal("1khz_d".into(), Data::Integer(21)))
                    .await
                    .unwrap();
                tokio::time::sleep(Duration::from_millis(1)).await;
            },
            _ => {}
        }
    }
}
