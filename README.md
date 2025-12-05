# Brainfuck 1:1 Chat Application

A complete implementation of a **1:1 chat application written in Brainfuck**, using the Trainfuck networking extensions.
[Demo](https://the.plot.is/wsr6J68z)

## Project Overview

This project consists of:

1. **A Trainfuck Interpreter** written in Rust - extends standard Brainfuck with TCP networking capabilities
2. **Chat Server** written in Trainfuck (`.bf` file) - listens for connections and echoes messages
3. **Chat Client** written in Trainfuck (`.bf` file) - connects to server, sends user input, displays responses

## What is Trainfuck?

[Trainfuck](https://imrannazar.com/articles/trainfuck) is an extension of the esoteric Brainfuck language that adds networking and file I/O capabilities while maintaining Brainfuck's minimalist philosophy.

### Standard Brainfuck Commands (8 commands)

| Command | Description                                |
| ------- | ------------------------------------------ |
| `>`     | Move data pointer right                    |
| `<`     | Move data pointer left                     |
| `+`     | Increment byte at pointer                  |
| `-`     | Decrement byte at pointer                  |
| `.`     | Output byte at pointer as ASCII            |
| `,`     | Input byte to pointer                      |
| `[`     | Jump past matching `]` if byte is 0        |
| `]`     | Jump back to matching `[` if byte is not 0 |

### Trainfuck Networking Extensions (5 commands)

| Command | Description                           |
| ------- | ------------------------------------- |
| `$`     | Listen on address/port (server mode)  |
| `%`     | Connect to address/port (client mode) |
| `@`     | Accept connection / Close connection  |
| `` ` `` | Receive byte from network             |
| `'`     | Send byte to network                  |

**Address/Port Format:** The IP address is read from 4 consecutive memory cells starting at the current pointer (big-endian IPv4), and the port from the next 2 cells (big-endian uint16).

## Building

```bash
# Clone and build
cargo build --release
```

## Usage

### Running the Chat Server

```bash
./target/release/trainfuck chat/server.bf
```

The server:

- Listens on `127.0.0.1:8888`
- Accepts one connection at a time
- Echoes back any received bytes
- Loops to accept new connections after disconnect

### Running the Chat Client

In a separate terminal:

```bash
./target/release/trainfuck chat/client.bf
```

The client:

- Connects to `127.0.0.1:8888`
- Reads from stdin
- Sends each character to server
- Prints received echoes

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Rust Interpreter                          │
│  ┌──────────┐    ┌──────────┐    ┌──────────────────────┐  │
│  │  Parser  │ -> │   VM     │ -> │  Networking Layer    │  │
│  │          │    │ (tape +  │    │  (TcpListener,       │  │
│  │ .bf file │    │ pointer) │    │   TcpStream)         │  │
│  └──────────┘    └──────────┘    └──────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                    Network                                   │
│                                                              │
│   Server (server.bf)  <──TCP──>  Client (client.bf)         │
│   Port 8888                                                  │
└─────────────────────────────────────────────────────────────┘
```

## Files

```
├── Cargo.toml
├── src/
│   ├── main.rs             # CLI entry point
│   └── interpreter.rs      # Trainfuck VM + parser
├── chat/
│   ├── server.bf
		├── hello.bf						# Hello World (standard BF)
│   └── client.bf           # Chat client in Trainfuck
└── README.md
```

## Technical Notes

### Interpreter Features

- **Optimized parsing**: Consecutive `+`, `-`, `>`, `<` are combined into single operations
- **30KB tape**: Standard Brainfuck memory size
- **Wrapping arithmetic**: Cell values wrap at 0/255
- **Error handling**: Clear messages for parse errors and runtime issues

### Networking Implementation

- Uses Rust's `std::net` for TCP
- Non-blocking would require additional commands; current implementation is blocking
- Connection state managed in the VM struct

## References

- [Original Trainfuck Specification](https://imrannazar.com/articles/trainfuck) by Imran Nazar
- [Brainfuck on Esolang Wiki](https://esolangs.org/wiki/Brainfuck)
- [Brainfuck++ with Sockets](https://github.com/Gr3atWh173/brainfuckplusplus)

## License

MIT

---

_Built for a case study demonstrating creative problem-solving with esoteric languages._
