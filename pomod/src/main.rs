use std::fs::remove_file;
use std::io::{ErrorKind, Read, Result, Write};
use std::os::unix::net::UnixListener;
use std::process::exit;
use std::time::Instant;

use pomolib::{ResponseCodes, SessionState, SessionStatusMessage, Transmittable};

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
        match stream.as_ref() {
            Ok(mut s) => {
                // Handle The Input and Route it to actions
                let mut input_buf = vec![];
                s.take(5).read_to_end(&mut input_buf)?;
                dbg!(&input_buf);
                let message = handle_request(&input_buf[0..5], &mut app);
                match s.write(&message.to_bytes()) {
                    Ok(_) => Ok(()),
                    Err(e) if e.kind() == ErrorKind::BrokenPipe => {
                        eprintln!("Sender Hang up!");
                        Ok(())
                    }
                    Err(e) => Err(e),
                }?
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
fn handle_request(request: &[u8], app: &mut Option<Session>) -> Box<dyn Transmittable> {
    let final_bytes = request[1..=4].try_into().expect("Oopsie");

    match request[0] {
        1_u8 => {
            println!(
                "Recieved Start Request for {} Seconds",
                u32::from_be_bytes(final_bytes)
            );
            start_handler(u32::from_be_bytes(final_bytes), app)
        }
        2_u8 => stop_handler(app),
        3_u8 => status_handler(app),
        _ => {
            eprintln!("Will Not handle Faulty request");
            Box::new(ResponseCodes::InvalidRequest)
        }
    }
}

fn start_handler(seconds: u32, app: &mut Option<Session>) -> Box<dyn Transmittable> {
    match app {
        Some(sess) => match sess.status() {
            SessionState::Working => {
                eprintln!("You are already Working!");
                Box::new(ResponseCodes::Success)
            }
            SessionState::Resting => {
                eprintln!("Aren't you a Giddy one?");
                Box::new(ResponseCodes::Success)
            }
        },
        None => {
            // Create A New App
            *app = Some(Session::new(seconds, 600, SessionState::Working));
            Box::new(ResponseCodes::Success)
        }
    }
}

fn status_handler(app: &mut Option<Session>) -> Box<dyn Transmittable> {
    match app {
        Some(sess) => {
            println!("{} Seconds left", sess.seconds_left());
            Box::new(SessionStatusMessage::new(
                sess.work_seconds,
                sess.rest_seconds,
                sess.seconds_left(),
                sess.status(),
            ))
        }
        None => {
            eprintln!("No Session Started");
            Box::new(ResponseCodes::NoSessionExists)
        }
    }
}

fn stop_handler(app: &mut Option<Session>) -> Box<dyn Transmittable> {
    match app {
        Some(sess) => {
            *app = None;
            Box::new(ResponseCodes::Success)
        }
        None => Box::new(ResponseCodes::NoSessionExists),
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
    work_seconds: u32,
    rest_seconds: u32,
    status: SessionState,
    session_start: Instant,
}

impl Session {
    fn new(work_seconds: u32, rest_seconds: u32, status: SessionState) -> Self {
        Self {
            work_seconds,
            rest_seconds,
            status,
            session_start: Instant::now(),
        }
    }

    fn seconds_left(&mut self) -> u32 {
        let total_secs = Instant::now().duration_since(self.session_start).as_secs() as u32;
        if total_secs < (self.work_seconds + self.rest_seconds) {
            if total_secs < self.work_seconds {
                return self.work_seconds - total_secs;
            }
            return (self.work_seconds + self.rest_seconds) - total_secs;
        } else {
            let remainder = total_secs % (self.work_seconds + self.rest_seconds);
            if remainder == 0 {
                // We just started over
                self.status = SessionState::Working;
                self.work_seconds
            } else if (1..self.work_seconds).contains(&remainder) {
                // In the middle of a work Session
                self.status = SessionState::Working;
                self.work_seconds - remainder
            } else {
                // Resting
                self.status = SessionState::Resting;
                self.work_seconds + self.rest_seconds - remainder
            }
        }
    }

    fn status(&self) -> SessionState {
        self.status
    }
}

impl Default for Session {
    fn default() -> Self {
        Self {
            work_seconds: 25 * 60,
            rest_seconds: 5 * 60,
            status: SessionState::Working,
            session_start: Instant::now(),
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
