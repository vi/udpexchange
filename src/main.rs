use std::{
    net::{SocketAddr, UdpSocket},
    time::{Duration, Instant},
};

use argh::FromArgs;
use const_lru::ConstLru;

const MAX_CLIENTS: usize = 64;
const BUFSIZE: usize = 4096;

/// Simple UDP service which replies to all other known clients
#[derive(FromArgs)]
struct Opts {
    /// socket address to bind UDP to
    #[argh(positional)]
    listen_addr: SocketAddr,

    /// timeout, in seconds, to expire clients.
    #[argh(
        option,
        short = 't',
        default = "Duration::from_secs(60)",
        from_str_fn(parse_duration)
    )]
    timeout: Duration,
}

fn parse_duration(x: &str) -> Result<Duration, String> {
    Ok(Duration::from_secs(
        x.parse::<u64>()
            .map_err(|_| "Invalid number of seconds".to_owned())?,
    ))
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let opts: Opts = argh::from_env();
    let udp = UdpSocket::bind(opts.listen_addr)?;
    let mut addrs = ConstLru::<SocketAddr, Instant, MAX_CLIENTS, u8>::new();
    let mut buf = [0u8; BUFSIZE];
    loop {
        let Ok((n_bytes, from)) = udp.recv_from(&mut buf) else {
            std::thread::sleep(Duration::from_millis(50));
            continue;
        };

        let now = Instant::now();
        let buf = &buf[0..n_bytes];
        addrs.insert(from, now);

        if !buf.is_empty() {
            for (&addr, &last_update) in addrs.iter() {
                if addr == from {
                    continue;
                }
                if now.duration_since(last_update) > opts.timeout {
                    continue;
                }
                let _ = udp.send_to(buf, addr);
            }
        } else {
            let _ = udp.send_to(buf, from);
        }
    }
}
