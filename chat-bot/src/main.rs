use std::env;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, Instant};

// Client -> Server
const PKT_ADMIN_JOIN: u8 = 0;
const PKT_ADMIN_UPDATE_FREQUENCY: u8 = 2;
const PKT_ADMIN_RCON: u8 = 5;

// Server -> Client
const PKT_SERVER_PROTOCOL: u8 = 103;
const PKT_SERVER_WELCOME: u8 = 104;
const PKT_SERVER_CHAT: u8 = 119;

const UPDATE_CHAT: u16 = 5;
const FREQ_AUTOMATIC: u16 = 0x40;

fn build_packet(kind: u8, payload: &[u8]) -> Vec<u8> {
    let size = (3 + payload.len()) as u16;
    let mut pkt = Vec::with_capacity(size as usize);
    pkt.extend_from_slice(&size.to_le_bytes());
    pkt.push(kind);
    pkt.extend_from_slice(payload);
    pkt
}

fn cstring(s: &str) -> Vec<u8> {
    let mut v = s.as_bytes().to_vec();
    v.push(0);
    v
}

fn send_join(stream: &mut TcpStream, password: &str, name: &str) -> io::Result<()> {
    let mut payload = Vec::new();
    payload.extend(cstring(password));
    payload.extend(cstring(name));
    payload.extend(cstring("1.0.0"));
    stream.write_all(&build_packet(PKT_ADMIN_JOIN, &payload))
}

fn send_subscribe_chat(stream: &mut TcpStream) -> io::Result<()> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&UPDATE_CHAT.to_le_bytes());
    payload.extend_from_slice(&FREQ_AUTOMATIC.to_le_bytes());
    stream.write_all(&build_packet(PKT_ADMIN_UPDATE_FREQUENCY, &payload))
}

fn send_rcon(stream: &mut TcpStream, cmd: &str) -> io::Result<()> {
    stream.write_all(&build_packet(PKT_ADMIN_RCON, &cstring(cmd)))
}

fn read_packet(stream: &mut TcpStream) -> io::Result<(u8, Vec<u8>)> {
    let mut buf2 = [0u8; 2];
    stream.read_exact(&mut buf2)?;
    let size = u16::from_le_bytes(buf2) as usize;

    let mut buf1 = [0u8; 1];
    stream.read_exact(&mut buf1)?;
    let kind = buf1[0];

    let payload_len = size.saturating_sub(3);
    let mut payload = vec![0u8; payload_len];
    if payload_len > 0 {
        stream.read_exact(&mut payload)?;
    }
    Ok((kind, payload))
}

// Reads a null-terminated string from a byte slice, advancing the offset.
fn read_cstring(data: &[u8], offset: &mut usize) -> String {
    let start = *offset;
    while *offset < data.len() && data[*offset] != 0 {
        *offset += 1;
    }
    let s = String::from_utf8_lossy(&data[start..*offset]).into_owned();
    if *offset < data.len() {
        *offset += 1; // consume null byte
    }
    s
}

fn run(addr: &str, password: &str, bot_name: &str, save_name: &str, save_interval: Duration) -> io::Result<()> {
    let mut stream = TcpStream::connect(addr)?;
    println!("Connected to {addr}");

    send_join(&mut stream, password, bot_name)?;

    // Handshake: wait for PROTOCOL then WELCOME
    loop {
        let (kind, _payload) = read_packet(&mut stream)?;
        match kind {
            PKT_SERVER_PROTOCOL => {}
            PKT_SERVER_WELCOME => {
                println!("Authenticated. Subscribing to chat...");
                send_subscribe_chat(&mut stream)?;
                break;
            }
            other => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("unexpected packet {other} during handshake"),
                ));
            }
        }
    }

    println!(
        "Ready. Listening for !pause / !unpause. Auto-saving as '{}' every {}m.",
        save_name,
        save_interval.as_secs() / 60
    );

    // Use a short read timeout so the save timer fires even when the server is quiet.
    stream.set_read_timeout(Some(Duration::from_secs(1)))?;

    let mut last_save = Instant::now();

    loop {
        // Check if a periodic save is due before blocking on the next packet.
        if last_save.elapsed() >= save_interval {
            println!("Auto-saving as '{save_name}'...");
            send_rcon(&mut stream, "say Saving game...")?;
            send_rcon(&mut stream, &format!("save {save_name}"))?;
            last_save = Instant::now();
        }

        let (kind, payload) = match read_packet(&mut stream) {
            Ok(pkt) => pkt,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut => {
                continue;
            }
            Err(e) => return Err(e),
        };

        if kind != PKT_SERVER_CHAT {
            continue;
        }

        // Chat packet layout:
        //   u8  action
        //   u8  dest_type
        //   u32 client_id
        //   str message (null-terminated)
        //   u64 data
        if payload.len() < 6 {
            continue;
        }
        let mut offset = 6; // skip action(1) + dest_type(1) + client_id(4)
        let message = read_cstring(&payload, &mut offset);
        let trimmed = message.trim();

        match trimmed.to_lowercase().as_str() {
            "!pause" => {
                println!("Received !pause — pausing server.");
                send_rcon(&mut stream, "pause")?;
            }
            "!unpause" => {
                println!("Received !unpause — unpausing server.");
                send_rcon(&mut stream, "unpause")?;
            }
            "!save" => {
                println!("Received !save — saving game.");
                send_rcon(&mut stream, "say \"Saving game...\"")?;
                send_rcon(&mut stream, &format!("save {save_name}"))?;
                last_save = Instant::now();
            }
            _ => {}
        }
    }
}

fn main() {
    let host = env::var("OPENTTD_HOST").unwrap_or_else(|_| "openttd-server".to_string());
    let port = env::var("OPENTTD_ADMIN_PORT").unwrap_or_else(|_| "3977".to_string());
    let password = env::var("OPENTTD_ADMIN_PASSWORD").expect("OPENTTD_ADMIN_PASSWORD must be set");
    let bot_name = env::var("BOT_NAME").unwrap_or_else(|_| "chat-bot".to_string());
    let save_name = env::var("SAVENAME")
        .unwrap_or_else(|_| "autosave_bot".to_string())
        .trim_end_matches(".sav")
        .to_string();
    let save_interval_mins: u64 = env::var("SAVE_INTERVAL_MINS")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .expect("SAVE_INTERVAL_MINS must be a positive integer");

    let addr = format!("{host}:{port}");
    let save_interval = Duration::from_secs(save_interval_mins * 60);

    loop {
        match run(&addr, &password, &bot_name, &save_name, save_interval) {
            Ok(_) => println!("Disconnected. Reconnecting in 5s..."),
            Err(e) => eprintln!("Error: {e}. Reconnecting in 5s..."),
        }
        thread::sleep(Duration::from_secs(5));
    }
}
