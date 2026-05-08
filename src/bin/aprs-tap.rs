use chrono::Local;
use clap::Parser;
use colored::Colorize;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

use aprs_tap::aprs::AprsPacket;

#[derive(Parser)]
#[command(name = "aprs-tap", about = "APRS-IS stream reader")]
struct Args {
    /// APRS-IS server hostname
    #[arg(short, long, default_value = "rotate.aprs2.net")]
    server: String,

    /// APRS-IS server port (14580 = filtered, use -f to set filter; 10152 = full unfiltered feed)
    #[arg(short, long, default_value_t = 14580)]
    port: u16,

    /// Your callsign (N0CALL for anonymous read-only)
    #[arg(short = 'u', long, default_value = "N0CALL")]
    callsign: String,

    /// APRS-IS passcode (-1 for read-only)
    #[arg(long, default_value = "-1")]
    passcode: String,

    /// Server-side filter string, e.g. "r/38.9/-77.0/100" for a 100 km radius
    #[arg(short, long)]
    filter: Option<String>,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    eprintln!("Connecting to {}:{}...", args.server, args.port);

    let stream = TcpStream::connect(format!("{}:{}", args.server, args.port))?;
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);

    let login = match &args.filter {
        Some(f) => format!(
            "user {} pass {} vers aprs-tap 0.1.0 filter {}\r\n",
            args.callsign, args.passcode, f
        ),
        None => format!(
            "user {} pass {} vers aprs-tap 0.1.0\r\n",
            args.callsign, args.passcode
        ),
    };
    writer.write_all(login.as_bytes())?;

    let mut buf = Vec::new();

    loop {
        buf.clear();

        let n = reader.read_until(b'\n', &mut buf)?;

        if n == 0 {
            break;
        }

        if buf.ends_with(b"\n") || buf.ends_with(b"\r") {
            buf.pop();
        }

        if buf.is_empty() {
            continue;
        }

        display_line(&String::from_utf8_lossy(&buf));
    }

    Ok(())
}

fn display_line(line: &str) {
    let now = Local::now().format("%H:%M:%S").to_string();

    if line.starts_with('#') {
        println!("{} {}", format!("[{}]", now).dimmed(), line.dimmed());
        return;
    }

    match AprsPacket::try_from(line) {
        Ok(packet) => packet.display(&now),
        Err(_) => println!("{} {}", format!("[{}]", now).dimmed(), line),
    }
}
