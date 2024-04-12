> Undertone is still in it's infancy, feel free to contribute!

# Undertone

Undertone is a peer-to-peer (P2P) communication and file-sharing application built using WebRTC and the Rust programming language. It allows users to securely connect and share files, code, or any other data directly between their devices without relying on a central server.

## Undertone About

Undertone is designed to provide a secure and decentralized way of sharing messages between peers. It leverages the power of WebRTC, a technology that enables real-time communication directly between devices without the need for intermediary servers or plugins.

The application is built using Rust, a systems programming language known for its performance, safety, and concurrency features. Rust's strong type system and ownership model help ensure memory safety and prevent common programming errors, making it an excellent choice for building secure and reliable applications.

## Features Planned

- **Peer-to-Peer Communication**: Undertone allows direct communication between peers without relying on a central server, ensuring privacy and security.
- **File Sharing**: Users can share files of any type and size directly with other peers, enabling efficient and secure file transfers.
- **Secure Connections**: All communication between peers is encrypted using industry-standard encryption protocols, ensuring data privacy and security. (with a secure mode available)
- **Cross-Platform Support**: Undertone is designed to work across different platforms and operating systems, providing a consistent experience for all users.

## Prerequisites

Before building and running Undertone, ensure that you have the following prerequisites installed:

- Rust (latest stable version)
- Cargo (Rust's package manager)
- WebRTC-compatible browser or environment (e.g., Google Chrome, Mozilla Firefox)

## Building

To build Undertone from source, follow these steps:

1. Clone the Undertone repository:

```
git clone https://github.com/m4yc3x/undertone.git
```

2. Navigate to the project directory:

```
cd undertone
```

3. Build the project using Cargo:

```
cargo build --release
```
   This will compile the Undertone application in release mode for optimal performance.

4. Run the application:

```
cargo run --release
```
   This will start the Undertone application, and you should see instructions on how to connect with other peers.

## Contributing

Contributions to Undertone are welcome! If you find any issues or have ideas for improvements, please open an issue or submit a pull request on the project's GitHub repository.

When contributing, please follow the project's coding style and guidelines, and ensure that your changes are well-documented and tested.

## License

Undertone is released under the [MIT License](https://mit-license.org/). You are free to use, modify, and distribute the software as long as you include the original copyright and license notice.