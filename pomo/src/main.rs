use clap::{Parser, Subcommand};
use pomolib::{Request, ResponseCodes, SessionStatusMessage, Transmittable, POMO_SOCKET};
use std::io::{Read, Result, Write};
use std::os::unix::net::UnixStream;


#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Start { work: u32, rest: u32 },

    Stop,
    Status,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut stream = UnixStream::connect(POMO_SOCKET)?;

    match &args.command {
        Commands::Start { work, .. } => {
            let message = Request::new(1, *work);
            stream.write(&message.to_bytes())?;
            stream.flush()?;
            let mut buff = vec![];
            // TODO: Handle Response
            stream.read_to_end(&mut buff)?;
            let message_len = buff[0] as usize;
            if message_len == 1 {
                let resp = ResponseCodes::from_bytes(&mut buff[1..=message_len]);
                match resp {
                    ResponseCodes::Success => {}
                    ResponseCodes::InvalidRequest => {
                        eprintln!("Invalid Request!")
                    }
                    ResponseCodes::NoSessionExists => unreachable!(),
                }
            } else {
                panic!("Error! Invalid Message Response")
            }
        }

        Commands::Stop => {
            let message = Request::new(2, 0);
            stream.write(&message.to_bytes())?;
            stream.flush()?;
            let mut buff = vec![];
            // TODO: Handle Response
            stream.read_to_end(&mut buff)?;
            let message_len = buff[0] as usize;
            if message_len == 1 {
                let resp = ResponseCodes::from_bytes(&mut buff[1..=message_len]);
                match resp {
                    ResponseCodes::Success => {}
                    ResponseCodes::InvalidRequest => {
                        eprintln!("Invalid Request!")
                    }
                    ResponseCodes::NoSessionExists => {
                        eprintln!("No Session to stop")
                    }
                }
            }
        }

        Commands::Status => {
            let message = Request::new(3, 0);
            stream.write(&message.to_bytes())?;
            stream.flush()?;
            let mut buff = vec![];
            // TODO: Handle Response
            stream.read_to_end(&mut buff)?;
            let message_len = buff[0] as usize;
            if message_len == 0x01 {
                let resp = ResponseCodes::from_bytes(&mut buff[1..=message_len]);
                match resp {
                    ResponseCodes::Success => unreachable!(),
                    ResponseCodes::InvalidRequest => {
                        eprintln!("Invalid Request!")
                    }
                    ResponseCodes::NoSessionExists => {
                        eprintln!("No Session to stop")
                    }
                }
            } else {
                let resp = SessionStatusMessage::from_bytes(&mut buff[1..=message_len]);
                print!(
                    "{0} {1} {2} {3}",
                    resp.state, resp.work_seconds, resp.rest_seconds, resp.time_remaining
                );
            }
        }
    };

    Ok(())
}
