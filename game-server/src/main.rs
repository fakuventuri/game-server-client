use common::{FromClientMessage, FromServerMessage};

use message_io::network::{Endpoint, NetEvent, Transport};
use message_io::node::{self};

use std::collections::HashMap;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;

const PORT: u16 = 8282;

fn main() {
    run_server(
        Transport::FramedTcp,
        ("0.0.0.0", PORT).to_socket_addrs().unwrap().next().unwrap(),
    )
}

struct ClientInfo {
    count: usize,
}

pub fn run_server(transport: Transport, addr: SocketAddr) {
    println!("Server started");

    let (handler, listener) = node::split::<()>();

    let mut clients: HashMap<Endpoint, ClientInfo> = HashMap::new();

    match handler.network().listen(transport, addr) {
        Ok((_id, real_addr)) => println!("Server running at {} by {}", real_addr, transport),
        Err(_) => return println!("Can not listening at {} by {}", addr, transport),
    }

    // Handle Ctrl-C to stop server
    {
        let stop_handler = handler.clone();
        ctrlc::set_handler(move || {
            stop_handler.stop();
            println!("Server stopped");
        })
        .expect("Error setting Ctrl-C handler");
    }

    listener.for_each(move |event| match event.network() {
        NetEvent::Connected(_, _) => (), // Only generated at connect() calls.
        NetEvent::Accepted(endpoint, _listener_id) => {
            // Only connection oriented protocols will generate this event
            clients.insert(endpoint, ClientInfo { count: 0 });
            println!(
                "Client ({}) connected (total clients: {})",
                endpoint.addr(),
                clients.len()
            );
        }
        NetEvent::Message(endpoint, input_data) => {
            let message: FromClientMessage = bincode::deserialize(&input_data).unwrap();
            match message {
                FromClientMessage::Ping => {
                    let message = match clients.get_mut(&endpoint) {
                        Some(client) => {
                            // For connection oriented protocols
                            client.count += 1;
                            println!("Ping from {}, {} times", endpoint.addr(), client.count);
                            FromServerMessage::Pong(client.count)
                        }
                        None => {
                            // For non-connection oriented protocols
                            println!("Ping from {}", endpoint.addr());
                            FromServerMessage::UnknownPong
                        }
                    };
                    let output_data = bincode::serialize(&message).unwrap();
                    handler.network().send(endpoint, &output_data);
                }
            }
        }
        NetEvent::Disconnected(endpoint) => {
            // Only connection oriented protocols will generate this event
            clients.remove(&endpoint).unwrap();
            println!(
                "Client ({}) disconnected (total clients: {})",
                endpoint.addr(),
                clients.len()
            );
        }
    });
}
