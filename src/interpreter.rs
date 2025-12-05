//! Trainfuck Interpreter
//!
//! ## Standard Brainfuck Commands
//! - `>` : Move pointer right
//! - `<` : Move pointer left
//! - `+` : Increment current cell
//! - `-` : Decrement current cell
//! - `.` : Output current cell as ASCII
//! - `,` : Input character to current cell
//! - `[` : Jump past matching `]` if cell is 0
//! - `]` : Jump back to matching `[` if cell is not 0
//!
//! ## Trainfuck Networking Extensions
//! - `%` : Connect to address/port (client mode)
//! - `$` : Listen on address/port (server mode)
//! - `@` : Accept incoming connection / close connection
//! - `` ` `` : Receive byte from network
//! - `'` : Send byte to network

use std::io::{self, BufRead, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use thiserror::Error;

/// Memory tape size (30KB as per original Brainfuck spec)
const TAPE_SIZE: usize = 30_000;

#[derive(Error, Debug)]
pub enum TrainfuckError {
    #[error("Unmatched '[' at position {0}")]
    UnmatchedOpenBracket(usize),

    #[error("Unmatched ']' at position {0}")]
    UnmatchedCloseBracket(usize),

    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    #[error("Network error: {0}")]
    NetworkError(String),
}

pub type Result<T> = std::result::Result<T, TrainfuckError>;

/// Represents parsed Trainfuck operations
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    // Standard Brainfuck
    MoveRight(usize), // Optimized: multiple > combined
    MoveLeft(usize),  // Optimized: multiple < combined
    Increment(u8),    // Optimized: multiple + combined
    Decrement(u8),    // Optimized: multiple - combined
    Output,
    Input,
    Loop(Vec<Op>),

    // Trainfuck Networking
    Connect, // %
    Listen,  // $
    Accept,  // @
    Receive, // `
    Send,    // '
}

/// Parses Trainfuck source code into operations
pub fn parse(source: &str) -> Result<Vec<Op>> {
    let chars: Vec<char> = source.chars().collect();
    let mut ops = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            '>' => {
                let count = count_consecutive(&chars, i, '>');
                ops.push(Op::MoveRight(count));
                i += count;
            }
            '<' => {
                let count = count_consecutive(&chars, i, '<');
                ops.push(Op::MoveLeft(count));
                i += count;
            }
            '+' => {
                let count = count_consecutive(&chars, i, '+');
                ops.push(Op::Increment((count % 256) as u8));
                i += count;
            }
            '-' => {
                let count = count_consecutive(&chars, i, '-');
                ops.push(Op::Decrement((count % 256) as u8));
                i += count;
            }
            '.' => {
                ops.push(Op::Output);
                i += 1;
            }
            ',' => {
                ops.push(Op::Input);
                i += 1;
            }
            '[' => {
                let (loop_ops, end_pos) = parse_loop(&chars, i)?;
                ops.push(Op::Loop(loop_ops));
                i = end_pos + 1;
            }
            ']' => {
                return Err(TrainfuckError::UnmatchedCloseBracket(i));
            }
            // Trainfuck networking
            '%' => {
                ops.push(Op::Connect);
                i += 1;
            }
            '$' => {
                ops.push(Op::Listen);
                i += 1;
            }
            '@' => {
                ops.push(Op::Accept);
                i += 1;
            }
            '`' => {
                ops.push(Op::Receive);
                i += 1;
            }
            '\'' => {
                ops.push(Op::Send);
                i += 1;
            }
            // Everything else is a comment
            _ => {
                i += 1;
            }
        }
    }

    Ok(ops)
}

fn count_consecutive(chars: &[char], start: usize, target: char) -> usize {
    chars[start..].iter().take_while(|&&c| c == target).count()
}

fn parse_loop(chars: &[char], start: usize) -> Result<(Vec<Op>, usize)> {
    let mut depth = 1;
    let mut i = start + 1;

    while i < chars.len() && depth > 0 {
        match chars[i] {
            '[' => depth += 1,
            ']' => depth -= 1,
            _ => {}
        }
        if depth > 0 {
            i += 1;
        }
    }

    if depth != 0 {
        return Err(TrainfuckError::UnmatchedOpenBracket(start));
    }

    // Parse the content between [ and ]
    let inner_source: String = chars[start + 1..i].iter().collect();
    let inner_ops = parse(&inner_source)?;

    Ok((inner_ops, i))
}

/// The Trainfuck virtual machine
pub struct VM {
    tape: Vec<u8>,
    pointer: usize,

    // Networking state
    listener: Option<TcpListener>,
    connection: Option<TcpStream>,

    // I/O streams
    pub input: Box<dyn BufRead>,
    pub output: Box<dyn Write>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            tape: vec![0u8; TAPE_SIZE],
            pointer: 0,
            listener: None,
            connection: None,
            input: Box::new(io::BufReader::new(io::stdin())),
            output: Box::new(io::stdout()),
        }
    }

    /// Execute parsed operations
    pub fn execute(&mut self, ops: &[Op]) -> Result<()> {
        for op in ops {
            self.execute_op(op)?;
        }
        Ok(())
    }

    fn execute_op(&mut self, op: &Op) -> Result<()> {
        match op {
            Op::MoveRight(n) => {
                self.pointer = self.pointer.wrapping_add(*n);
                if self.pointer >= TAPE_SIZE {
                    self.pointer = self.pointer % TAPE_SIZE;
                }
            }
            Op::MoveLeft(n) => {
                if *n > self.pointer {
                    // Wrap around
                    self.pointer = TAPE_SIZE - (*n - self.pointer);
                } else {
                    self.pointer -= *n;
                }
            }
            Op::Increment(n) => {
                self.tape[self.pointer] = self.tape[self.pointer].wrapping_add(*n);
            }
            Op::Decrement(n) => {
                self.tape[self.pointer] = self.tape[self.pointer].wrapping_sub(*n);
            }
            Op::Output => {
                let c = self.tape[self.pointer];
                self.output.write_all(&[c])?;
                self.output.flush()?;
            }
            Op::Input => {
                let mut buf = [0u8; 1];
                match self.input.read(&mut buf) {
                    Ok(0) => self.tape[self.pointer] = 0,
                    Ok(_) => self.tape[self.pointer] = buf[0],
                    Err(e) => return Err(TrainfuckError::IoError(e)),
                }
            }
            Op::Loop(inner_ops) => {
                while self.tape[self.pointer] != 0 {
                    self.execute(inner_ops)?;
                }
            }

            // Networking operations
            Op::Listen => self.net_listen()?,
            Op::Accept => self.net_accept()?,
            Op::Connect => self.net_connect()?,
            Op::Receive => self.net_receive()?,
            Op::Send => self.net_send()?,
        }
        Ok(())
    }

    /// Listen on address:port from tape
    /// Address: 4 bytes at pointer (big-endian IPv4)
    /// Port: 2 bytes at pointer+4 (big-endian)
    fn net_listen(&mut self) -> Result<()> {
        if self.listener.is_some() {
            // Already listening, close existing
            self.listener = None;
            return Ok(());
        }

        let addr = self.read_address_from_tape();
        let port = self.read_port_from_tape();

        let socket_addr = SocketAddrV4::new(addr, port);
        let listener = TcpListener::bind(socket_addr)
            .map_err(|e| TrainfuckError::NetworkError(format!("Failed to bind: {}", e)))?;

        eprintln!("[trainfuck] Listening on {}:{}", addr, port);
        self.listener = Some(listener);
        Ok(())
    }

    /// Accept incoming connection
    fn net_accept(&mut self) -> Result<()> {
        if self.connection.is_some() {
            // Close existing connection
            self.connection = None;
            eprintln!("[trainfuck] Connection closed");
            return Ok(());
        }

        if let Some(ref listener) = self.listener {
            let (stream, peer) = listener
                .accept()
                .map_err(|e| TrainfuckError::NetworkError(format!("Accept failed: {}", e)))?;
            eprintln!("[trainfuck] Accepted connection from {}", peer);
            self.connection = Some(stream);
        }
        Ok(())
    }

    /// Connect to address:port from tape
    fn net_connect(&mut self) -> Result<()> {
        if self.connection.is_some() {
            // Already connected, close
            self.connection = None;
            return Ok(());
        }

        let addr = self.read_address_from_tape();
        let port = self.read_port_from_tape();

        let socket_addr = SocketAddrV4::new(addr, port);
        let stream = TcpStream::connect(socket_addr)
            .map_err(|e| TrainfuckError::NetworkError(format!("Connect failed: {}", e)))?;

        eprintln!("[trainfuck] Connected to {}:{}", addr, port);
        self.connection = Some(stream);
        Ok(())
    }

    /// Receive a byte from network, store at pointer
    fn net_receive(&mut self) -> Result<()> {
        if let Some(ref mut stream) = self.connection {
            let mut buf = [0u8; 1];
            match stream.read(&mut buf) {
                Ok(0) => {
                    // Connection closed
                    self.tape[self.pointer] = 0;
                }
                Ok(_) => {
                    self.tape[self.pointer] = buf[0];
                }
                Err(e) => {
                    eprintln!("[trainfuck] Receive error: {}", e);
                    self.tape[self.pointer] = 0;
                }
            }
        } else {
            self.tape[self.pointer] = 0;
        }
        Ok(())
    }

    /// Send byte at pointer to network
    fn net_send(&mut self) -> Result<()> {
        if let Some(ref mut stream) = self.connection {
            let byte = self.tape[self.pointer];
            stream
                .write_all(&[byte])
                .map_err(|e| TrainfuckError::NetworkError(format!("Send failed: {}", e)))?;
            stream.flush()?;
        }
        Ok(())
    }

    /// Read IPv4 address from tape at pointer position
    fn read_address_from_tape(&self) -> Ipv4Addr {
        Ipv4Addr::new(
            self.tape[self.pointer],
            self.tape[self.pointer + 1],
            self.tape[self.pointer + 2],
            self.tape[self.pointer + 3],
        )
    }

    /// Read port from tape at pointer+4 position (big-endian)
    fn read_port_from_tape(&self) -> u16 {
        ((self.tape[self.pointer + 4] as u16) << 8) | (self.tape[self.pointer + 5] as u16)
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}
