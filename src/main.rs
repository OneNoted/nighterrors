mod backends;
mod cli;
mod color;
mod daemon;
mod ipc;
mod output;
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
        Command::Help(topic) => {
            println!("{}", cli::usage_for(topic));
            Ok(())
        }
        Command::Version => {
            println!("nighterrors {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Command::Run(options) => daemon::run(options, parsed.socket),
        Command::Control(request) => run_control_command(parsed.socket, parsed.output_mode, request),
    }
}

fn run_control_command(
    socket_override: Option<std::path::PathBuf>,
    output_mode: cli::OutputMode,
    request: ControlRequest,
) -> Result<(), String> {
    let socket_path = socket_override.unwrap_or_else(ipc::default_socket_path);
    let response = ipc::send_request(&socket_path, &request.to_wire())
        .map_err(|err| add_connect_hint(err, &socket_path))?;
    let rendered = output::render_response(&request, &response, output_mode, stdout_is_tty());
    println!("{rendered}");
    Ok(())
}

fn stdout_is_tty() -> bool {
    unsafe { libc::isatty(libc::STDOUT_FILENO) == 1 }
}

fn add_connect_hint(err: String, socket_path: &std::path::Path) -> String {
    if err.starts_with("failed to connect to") {
        format!(
            "{err}\nhint: start daemon with: nighterrors run (socket: {})",
            socket_path.display()
        )
    } else {
        err
    }
}
