use std::fs::remove_file;
use std::io::{Read, Result};
use std::os::unix::net::UnixListener;
use std::process::exit;
use std::time::Instant;

const POMO_SOCKET: &str = "/var/run/pomod.sock";

fn main() -> Result<()> {
    let _ = ctrlc::set_handler(|| {
        eprintln!("Recieved Keyboard Interupt!");
        match remove_file(POMO_SOCKET) {
            Ok(_) => (),
            Err(_) => eprintln!("Unable to remove Socket {}", POMO_SOCKET),
        };
        exit(1)
    });

    let socket = UnixListener::bind(POMO_SOCKET)?;

    let mut app: Option<Session> = None;

    for stream in socket.incoming() {
        match stream {
            Ok(mut s) => {
                // Handle The Input and Route it to actions
                let mut input_buf = vec![];
                s.read_to_end(&mut input_buf)?;
                dbg!(&input_buf);
                handle_request(&input_buf[0..5], &mut app);
            }
            Err(_) => {
                eprintln!("Socket Connection Failed")
            }
        }
    }
    Ok(())
}

/// Parse Request
///
/// Pomod Protocol: 1 byte -> Command, 4 bytes-> Optional data to use. If command doesn't
/// Take additional data, then push only 0x0
/// And then route it to handler
fn handle_request(request: &[u8], app: &mut Option<Session>) {
    let final_bytes = request[1..=4].try_into().expect("Oopsie");

    match request[0] {
        1_u8 => {
            println!(
            "Recieved Start Request for {} Seconds",
            u32::from_be_bytes(final_bytes)
        );
            start_handler(u32::from_be_bytes(final_bytes), app)
        },
        2_u8 => todo!("Handle Stop"),
        3_u8 => todo!("Handle Status"),
        _ => eprintln!("Will Not handle Faulty request"),
    }
}

fn start_handler(seconds: u32, app: &mut Option<Session>) {
    match app {
        Some(sess) => {
            match sess.status {
                SessionStatus::Working => eprintln!("You are already Working!"),
                SessionStatus::Resting => eprintln!("Aren't you a a Giddy one?"),
            }
        }
        None => {
            let chunk = TimeChunk::new(seconds);
            // Create A New App
            *app = Some(Session::new(SessionStatus::Working, chunk));
        }
    }
}

/// What we want to do with the Pomodoro
#[derive(Debug, Clone, Copy)]
#[repr(i8)]
enum Commands {
    Start(u32) = 1,
    Stop = 2,
    Status = 3,
}

#[derive(Debug, Clone, Copy)]
struct Session {
    status: SessionStatus,
    current_chunk: TimeChunk,
}

impl Session {
    fn new(status: SessionStatus, current_chunk: TimeChunk) -> Self {
        Self {
            status,
            current_chunk,
        }
    }
}

impl Default for Session {
    fn default() -> Self {
        Self {
            status: SessionStatus::Working,
            current_chunk: TimeChunk::new(25 * 60),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum SessionStatus {
    Working,
    Resting,
}

/// A TimeChunk represents a set of time
/// That is used keep track.
#[derive(Debug, Clone, Copy)]
struct TimeChunk {
    /// The number of Seconds in the Time frame
    span_seconds: u32,
    /// The time the chunk Started
    start_time: Instant,
}

impl TimeChunk {
    fn new(span_seconds: u32) -> Self {
        Self {
            span_seconds,
            start_time: Instant::now(),
        }
    }
}
