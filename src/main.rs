
#![cfg_attr(feature="mini", no_main)]


use std::{
    net::{SocketAddr, UdpSocket},
    time::{Duration, Instant}, process::exit,
};

use argh::FromArgs;
use const_lru::ConstLru;
#[cfg(feature="mini")]
use libc::{c_void,c_int, c_char};
use static_alloc::Bump;

const MAX_CLIENTS: usize = 64;
const BUFSIZE: usize = 4096;

#[global_allocator]
static A: Bump<[u8; 4096]> = Bump::uninit();

#[allow(unused)]
const MAX_REPLAY: usize = 16;

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

    #[cfg(feature="replay")]
    /// send recent accumulated messages to newly seen clients
    #[argh(
        switch,
        short='r',
    )]
    replay: bool,
}

fn parse_duration(x: &str) -> Result<Duration, String> {
    Ok(Duration::from_secs(
        x.parse::<u64>()
            .map_err(|_| "Invalid number of seconds".to_owned())?,
    ))
}


#[cfg(not(feature="mini"))]
fn msg(x: &[u8]) {
    use std::io::Write;
    let _ = std::io::stderr().write_all(x);
}

#[cfg(feature="mini")]
fn msg(x: &[u8]) {
    let _  = unsafe { libc::write(2, x as *const [u8] as *const c_void, x.len()) };
}

type Buffer = [u8; BUFSIZE];
fn run(opts: Opts) {
    let Ok(udp) = UdpSocket::bind(opts.listen_addr) else {
        msg(b"Failed to bind or listen the UDP socket\n");
        exit(1);
    };
    let mut addrs = ConstLru::<SocketAddr, Instant, MAX_CLIENTS, u8>::new();
    let mut recv_buffer : Buffer = [0; BUFSIZE];
    #[cfg(feature="replay")]
    let mut replays = ConstLru::<Instant, (Buffer, usize), MAX_REPLAY, u8>::new();
    loop {
        let Ok((n_bytes, from)) = udp.recv_from(&mut recv_buffer) else {
            std::thread::sleep(Duration::from_millis(50));
            continue;
        };

        let now = Instant::now();
        let buf = &recv_buffer[0..n_bytes];
        if addrs.insert(from, now) == None {
            #[cfg(feature="replay")]
            if opts.replay {
                for (&data_time, &(data, data_len)) in replays.iter() {
                    if now.duration_since(data_time) > opts.timeout {
                        continue;
                    }
                    let _ = udp.send_to(&data[..data_len], from);
                }
            }
        }

        if !buf.is_empty() {
            #[cfg(feature="replay")]
            if opts.replay {
                replays.insert(now, (recv_buffer, n_bytes));
            }
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

#[cfg(not(feature="mini"))]
fn main() {
    let opts: Opts = argh::from_env();
    run(opts)
}

#[cfg(feature="mini")]
#[no_mangle]
fn main(mut argc: c_int, argv: *mut*mut c_char, _envp: *mut*mut c_char) -> c_int {
    const MAXARGS : usize = 4;
    let mut x = [""; MAXARGS];
    if argc > MAXARGS as c_int{
        argc = MAXARGS as c_int;
    }
    for i in 1..argc {
        let v : *const c_char = unsafe { *argv.offset(i as isize) };
        let l : usize = unsafe { libc::strlen(v) as usize };
        let v = v as *const u8;
        let v : &'static [u8] = unsafe { std::slice::from_raw_parts(v, l) };
        let v : &str = unsafe { std::str::from_utf8_unchecked(v) };
        x[(i-1) as usize] = v;
    }
    match Opts::from_args(&["udpexchange"], &x[..((argc-1) as usize)]) {
        Ok(x) => {run(x); 0}
        Err(e) => {
            msg(e.output.as_bytes());
            1
        },
    }
}
