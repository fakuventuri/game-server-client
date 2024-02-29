use common::{FromClientMessage, FromServerMessage};

use message_io::network::{NetEvent, RemoteAddr, ToRemoteAddr, Transport};
use message_io::node::{self, NodeEvent};

use std::time::Duration;

// const PORT: u16 = 8282;

fn main() {
    run_client(
        Transport::FramedTcp,
        "127.0.0.1:8282".to_remote_addr().unwrap(),
    )
}

enum Signal {
    Greet, // This is a self event called every second.
           // Other signals here,
}

pub fn run_client(transport: Transport, remote_addr: RemoteAddr) {
    println!("Client started");

    let (handler, listener) = node::split();

    let (server_id, local_addr) = handler
        .network()
        .connect(transport, remote_addr.clone())
        .unwrap();

    // Handle Ctrl-C to stop client
    {
        let stop_handler = handler.clone();
        ctrlc::set_handler(move || {
            stop_handler.stop();
            println!("Client stopped");
        })
        .expect("Error setting Ctrl-C handler");
    }

    listener.for_each(move |event| match event {
        NodeEvent::Network(net_event) => match net_event {
            NetEvent::Connected(_, established) => {
                if established {
                    println!(
                        "Connected to server at {} by {}",
                        server_id.addr(),
                        transport
                    );
                    println!("Client identified by local port: {}", local_addr.port());
                    handler.signals().send(Signal::Greet);
                } else {
                    println!(
                        "Can not connect to server at {} by {}",
                        remote_addr, transport
                    );
                    handler.stop();
                }
            }
            NetEvent::Accepted(_, _) => unreachable!(), // Only generated when a listener accepts
            NetEvent::Message(_, input_data) => {
                let message: FromServerMessage = bincode::deserialize(&input_data).unwrap();
                match message {
                    FromServerMessage::Pong(count) => {
                        println!("Pong from server: {} times", count)
                    }
                    FromServerMessage::UnknownPong => println!("Pong from server"),
                }
            }
            NetEvent::Disconnected(_) => {
                println!("Server is disconnected");
                handler.stop();
            }
        },
        NodeEvent::Signal(signal) => match signal {
            Signal::Greet => {
                let message = FromClientMessage::Ping;
                let output_data = bincode::serialize(&message).unwrap();
                handler.network().send(server_id, &output_data);
                handler
                    .signals()
                    .send_with_timer(Signal::Greet, Duration::from_secs(1));
            }
        },
    });
}
