use std::process::exit;
use std::time::Instant;
use std::io::{Result, Read};
use std::os::unix::net::UnixListener;
use std::fs::remove_file;

const POMO_SOCKET: &str =  "/var/run/pomod.sock";

fn main() -> Result<()>{

    let _ = ctrlc::set_handler(|| {
        eprintln!("Recieved Keyboard Interupt!");
        match remove_file(POMO_SOCKET) {
            Ok(_) => (),
            Err(_) => eprintln!("Unable to remove Socket {}", POMO_SOCKET),
        };
        exit(1)
    });

    let socket = UnixListener::bind(POMO_SOCKET)?;

    for stream in socket.incoming() {
        match stream {
            Ok(mut s) => {
                // Handle The Input and Route it to actions
                let mut input_buf = String::new();
                s.read_to_string(&mut input_buf)?;
                dbg!(input_buf);
            }
            Err(_) => {
                eprintln!("Socket Connection Failed")
            }
        }
    }
    Ok(())
}


#[derive(Debug, Clone, Copy)]
struct Session {
    status: SessionStatus,
    current_chunk: TimeChunk,
}

impl Default for Session {

    fn default() -> Self {
        Self { status: SessionStatus::Working, current_chunk: TimeChunk::new(25 * 60) }
    }

}



#[derive(Debug, Clone, Copy)]
enum SessionStatus{
    Working,
    Resting
}


/// A TimeChunk represents a set of time
/// That is used keep track.
#[derive(Debug, Clone, Copy)]
struct TimeChunk {
    /// The number of Seconds in the Time frame
    span_seconds: usize,
    /// The time the chunk Started
    start_time: Instant,
}

impl TimeChunk {
    fn new(span_seconds: usize) -> Self {
        Self {
            span_seconds,
            start_time: Instant::now(),
        }
    }
}
