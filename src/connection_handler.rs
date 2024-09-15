use std::collections::HashSet;

use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::broadcast::*;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_util::codec::Framed;

use crate::{
    codec::MessageCodec,
    message::{Data, Message},
};

pub async fn start_connection<IO>(io: Framed<IO, MessageCodec>, bus: Sender<(String, Data)>)
where
    IO: AsyncRead + AsyncWrite + Send + 'static,
{
    let (sub_tx, sub_rx) = unbounded_channel::<String>();
    tokio::spawn(async move {
        let (sink, stream) = io.split();
        let stream_bus = bus.clone();
        let sink_bus = bus.subscribe();

        let handle_a = tokio::spawn(async move {
            start_sink(sink, sink_bus, sub_rx).await;
        });
        let abort_a = handle_a.abort_handle();
        let handle_b = tokio::spawn(async move {
            start_stream(stream, stream_bus, sub_tx).await;
        });
        let abort_b = handle_b.abort_handle();

        tokio::select! {
            _ = handle_a => {
                abort_b.abort();
            }
            _ = handle_b => {
                abort_a.abort();
            }
        }
    });
}

pub async fn start_sink<IO>(
    mut io: SplitSink<Framed<IO, MessageCodec>, Message>,
    mut bus: Receiver<(String, Data)>,
    mut sub_rx: UnboundedReceiver<String>,
) where
    IO: AsyncRead + AsyncWrite + Send + 'static,
{
    let mut subscriptions = HashSet::new();
    loop {
        tokio::select! {
            Some(subscription) = sub_rx.recv() => {
                subscriptions.insert(subscription);
            }
            Ok((name, data)) = bus.recv() => {
                if subscriptions.contains(&name) {
                    io.send(Message::Signal(name, data)).await.unwrap();
                }
            }
        }
    }
}

pub async fn start_stream<IO>(
    mut io: SplitStream<Framed<IO, MessageCodec>>,
    bus: Sender<(String, Data)>,
    sub_tx: UnboundedSender<String>,
) where
    IO: AsyncRead + AsyncWrite + Send + 'static,
{
    while let Some(Ok(message)) = io.next().await {
        match message {
            Message::Signal(name, data) => {
                bus.send((name, data)).unwrap();
            }
            Message::Subscription(name) => {
                sub_tx.send(name).unwrap();
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::*;
    use tokio::time::timeout;
    use tokio::{io::duplex, sync::broadcast::channel};
    use tokio_util::codec::Framed;

    use crate::codec::MessageCodec;

    #[tokio::test]
    pub async fn test_subscription() {
        let (server, client) = duplex(12_000);

        let (bus, _) = channel(1000);
        let server_framed = Framed::new(server, MessageCodec::new());
        start_connection(server_framed, bus).await;

        let mut client_framed = Framed::new(client, MessageCodec::new());

        client_framed
            .send(Message::Subscription("hi".to_string()))
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;

        client_framed
            .send(Message::Signal("hi".to_string(), Data::Integer(1)))
            .await
            .unwrap();
        if let Ok(Some(Ok(response))) =
            timeout(Duration::from_millis(100), client_framed.next()).await
        {
            assert_eq!(
                response,
                Message::Signal("hi".to_string(), Data::Integer(1))
            );
        } else {
            panic!("Did not receive response");
        }
    }

    #[tokio::test]
    pub async fn test_no_subscription() {
        let (server, client) = duplex(12_000);

        let (bus, _) = channel(1000);
        let server_framed = Framed::new(server, MessageCodec::new());
        start_connection(server_framed, bus).await;

        let mut client_framed = Framed::new(client, MessageCodec::new());

        client_framed
            .send(Message::Signal("hi".to_string(), Data::Integer(1)))
            .await
            .unwrap();
        if let Ok(Some(Ok(_))) =
            timeout(Duration::from_millis(100), client_framed.next()).await
        {
            panic!("not supposed to receive anything");
        }
    }

    #[tokio::test]
    pub async fn test_mix_subscription() {
        let (server, client) = duplex(12_000);

        let (bus, _) = channel(1000);
        let server_framed = Framed::new(server, MessageCodec::new());
        start_connection(server_framed, bus).await;

        let mut client_framed = Framed::new(client, MessageCodec::new());

        client_framed
            .send(Message::Subscription("hi".to_string()))
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;

        client_framed
            .send(Message::Signal("bye".to_string(), Data::Float(12.0)))
            .await
            .unwrap();
        client_framed
            .send(Message::Signal("hi".to_string(), Data::Integer(1)))
            .await
            .unwrap();
        if let Ok(Some(Ok(response))) =
            timeout(Duration::from_millis(100), client_framed.next()).await
        {
            assert_eq!(
                response,
                Message::Signal("hi".to_string(), Data::Integer(1))
            );
        } else {
            panic!("Did not receive response");
        }

        if let Ok(Some(Ok(_))) =
            timeout(Duration::from_millis(200), client_framed.next()).await
        {
            panic!("not supposed to receive anything");
        }
    }
}
