use clap::{Parser, Subcommand};
use pomolib::{Request, Transmittable};
use std::io::{Read, Result, Write};
use std::os::unix::net::UnixStream;

const POMO_SOCKET: &str = "/var/run/pomod.sock";

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
        }

        Commands::Stop => {
            let message = Request::new(2, 0);
            stream.write(&message.to_bytes())?;
            stream.flush()?;
            let mut buff = vec![];
            // TODO: Handle Response
            stream.read_to_end(&mut buff)?;
        }

        Commands::Status => {
            let message = Request::new(3, 0);
            stream.write(&message.to_bytes())?;
            stream.flush()?;
            let mut buff = vec![];
            // TODO: Handle Response
            stream.read_to_end(&mut buff)?;
        }
    };

    Ok(())
}
