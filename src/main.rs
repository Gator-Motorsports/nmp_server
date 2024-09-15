use std::{net::SocketAddr, path::PathBuf};

use clap::Parser;
use codec::MessageCodec;
use connection_handler::start_connection;
use message::Data;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpSocket, UnixSocket},
    sync::broadcast::{channel, Sender},
};
use tokio_util::{codec::Framed, net::Listener};

mod codec;
mod connection_handler;
mod message;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    unix_path: Option<PathBuf>,
    tcp_addr: Option<SocketAddr>,
}

async fn start_listener<
    IO: AsyncRead + AsyncWrite + Send + 'static,
    A,
    L: Listener<Io = IO, Addr = A>,
>(
    mut listener: L,
    bus: Sender<(String, Data)>,
) {
    while let Ok((conn, _)) = listener.accept().await {
        let framed_connection = Framed::new(conn, MessageCodec::new());
        start_connection(framed_connection, bus.clone()).await;
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Args::parse();

    let (tx, _) = channel(1_000);

    if let Some(path) = args.unix_path {
        let socket = UnixSocket::new_stream().unwrap();
        socket.bind(path).unwrap();

        let listener = socket.listen(32).unwrap();

        let bus = tx.clone();
        tokio::spawn(async move {
            start_listener(listener, bus).await;
        });
    }

    if let Some(addr) = args.tcp_addr {
        let socket = TcpSocket::new_v4().unwrap();
        socket.bind(addr).unwrap();

        let listener = socket.listen(32).unwrap();
        let bus = tx.clone();
        tokio::spawn(async move {
            start_listener(listener, bus).await;
        });
    }
}
