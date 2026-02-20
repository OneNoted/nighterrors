mod backends;
mod cli;
mod color;
mod daemon;
mod ipc;
mod protocols;

use cli::{Command, ControlRequest};

fn main() {
    if let Err(err) = real_main() {
        if err.starts_with("error:") {
            eprintln!("{err}");
        } else {
            eprintln!("error: {err}");
        }
        std::process::exit(1);
    }
}

fn real_main() -> Result<(), String> {
    let parsed = cli::parse_args(std::env::args_os())?;

    match parsed.command {
        Command::Help => {
            println!("{}", cli::usage());
            Ok(())
        }
        Command::Version => {
            println!("nighterrors {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Command::Run(options) => daemon::run(options, parsed.socket),
        Command::Control(request) => run_control_command(parsed.socket, request),
    }
}

fn run_control_command(
    socket_override: Option<std::path::PathBuf>,
    request: ControlRequest,
) -> Result<(), String> {
    let socket_path = socket_override.unwrap_or_else(ipc::default_socket_path);
    let response = ipc::send_request(&socket_path, &request.to_wire())?;
    println!("{response}");
    Ok(())
}
