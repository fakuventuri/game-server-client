use message_io::network::{NetEvent, Transport};
use message_io::node::{self};

fn main() {
    println!("Starting server");

    // Create a node, the main message-io entity. It is divided in 2 parts:
    // The 'handler', used to make actions (connect, send messages, signals, stop the node...)
    // The 'listener', used to read events from the network or signals.
    let (handler, listener) = node::split::<()>();

    // Listen for TCP, UDP and WebSocket messages at the same time.
    handler
        .network()
        .listen(Transport::FramedTcp, "0.0.0.0:3042")
        .unwrap();
    // handler.network().listen(Transport::Udp, "0.0.0.0:3043").unwrap();
    // handler.network().listen(Transport::Ws, "0.0.0.0:3044").unwrap();

    // Read incoming network events.
    listener.for_each(move |event| match event.network() {
        NetEvent::Connected(_, _) => unreachable!(), // Used for explicit connections.
        NetEvent::Accepted(_endpoint, _listener) => println!("Client connected"), // Tcp or Ws
        NetEvent::Message(endpoint, data) => {
            println!("Received: {}", String::from_utf8_lossy(data));
            handler.network().send(endpoint, data);
        }
        NetEvent::Disconnected(_endpoint) => println!("Client disconnected"), //Tcp or Ws
    });
}
