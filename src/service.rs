use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use crate::cli::{ServiceCommand, ServiceInstallOptions};

const UNIT_NAME: &str = "nighterrors.service";

pub fn run(command: ServiceCommand) -> Result<(), String> {
    match command {
        ServiceCommand::Install(options) => install(options),
        ServiceCommand::Uninstall => uninstall(),
        ServiceCommand::Status => run_unit_command("status"),
        ServiceCommand::Start => run_unit_command("start"),
        ServiceCommand::Stop => run_unit_command("stop"),
        ServiceCommand::Restart => run_unit_command("restart"),
    }
}

fn install(options: ServiceInstallOptions) -> Result<(), String> {
    let unit_dir = user_unit_dir()?;
    fs::create_dir_all(&unit_dir).map_err(|err| {
        format!(
            "failed to create user systemd unit directory {}: {err}",
            unit_dir.display()
        )
    })?;

    let unit_path = unit_dir.join(UNIT_NAME);
    let unit_contents = render_unit_file(&options)?;
    fs::write(&unit_path, unit_contents)
        .map_err(|err| format!("failed to write unit file {}: {err}", unit_path.display()))?;

    run_systemctl_user(["daemon-reload"])?;
    run_systemctl_user(["enable", "--now", UNIT_NAME])?;

    println!(
        "Installed and enabled {} at {}",
        UNIT_NAME,
        unit_path.display()
    );
    println!("Check status with: systemctl --user status {}", UNIT_NAME);
    Ok(())
}

fn uninstall() -> Result<(), String> {
    let unit_path = user_unit_dir()?.join(UNIT_NAME);

    let disable_result = run_systemctl_user(["disable", "--now", UNIT_NAME]);
    if let Err(err) = disable_result
        && !is_missing_unit_error(&err)
    {
        return Err(err);
    }

    if unit_path.exists() {
        fs::remove_file(&unit_path)
            .map_err(|err| format!("failed to remove unit file {}: {err}", unit_path.display()))?;
    }

    run_systemctl_user(["daemon-reload"])?;
    println!("Uninstalled {}", UNIT_NAME);
    Ok(())
}

fn run_unit_command(action: &str) -> Result<(), String> {
    let output = run_systemctl_user([action, UNIT_NAME])?;
    match action {
        "status" => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stdout.trim().is_empty() {
                println!("{}", stdout.trim_end());
            }
            if !stderr.trim().is_empty() {
                eprintln!("{}", stderr.trim_end());
            }
        }
        "start" => println!("Started {}", UNIT_NAME),
        "stop" => println!("Stopped {}", UNIT_NAME),
        "restart" => println!("Restarted {}", UNIT_NAME),
        _ => {}
    }
    Ok(())
}

fn run_systemctl_user<I, S>(args: I) -> Result<Output, String>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    run_command(
        "systemctl",
        std::iter::once("--user".into()).chain(args.into_iter().map(Into::into)),
    )
}

fn run_command<I, S>(program: &str, args: I) -> Result<Output, String>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let mut command = Command::new(program);
    let argv: Vec<OsString> = args.into_iter().map(Into::into).collect();
    command.args(&argv);

    let output = command
        .output()
        .map_err(|err| format!("failed to execute `{program}`: {err}"))?;
    if output.status.success() {
        return Ok(output);
    }

    Err(format_command_error(program, &output))
}

fn format_command_error(program: &str, output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut message = format!(
        "{program} exited with status {}",
        output.status.code().unwrap_or(-1)
    );

    if !stdout.trim().is_empty() {
        message.push_str(&format!("\nstdout:\n{}", stdout.trim()));
    }
    if !stderr.trim().is_empty() {
        message.push_str(&format!("\nstderr:\n{}", stderr.trim()));
    }
    if message.contains("Failed to connect to bus") {
        message.push_str(
            "\nhint: ensure a user systemd session is available and run this from a graphical login.",
        );
    }

    message
}

fn is_missing_unit_error(err: &str) -> bool {
    let lowered = err.to_ascii_lowercase();
    lowered.contains("not loaded")
        || lowered.contains("does not exist")
        || lowered.contains("no such file")
        || lowered.contains("unit nighterrors.service not found")
}

fn user_unit_dir() -> Result<PathBuf, String> {
    let config_home = std::env::var("XDG_CONFIG_HOME").ok();
    let home = std::env::var("HOME").ok();
    user_unit_dir_from(config_home.as_deref(), home.as_deref())
}

fn user_unit_dir_from(config_home: Option<&str>, home: Option<&str>) -> Result<PathBuf, String> {
    if let Some(config_home) = config_home
        && !config_home.is_empty()
    {
        return Ok(PathBuf::from(config_home).join("systemd").join("user"));
    }

    let home = home
        .ok_or_else(|| "cannot resolve HOME or XDG_CONFIG_HOME for user unit path".to_string())?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("systemd")
        .join("user"))
}

fn render_unit_file(options: &ServiceInstallOptions) -> Result<String, String> {
    let exec_start = render_exec_start(options)?;
    Ok(format!(
        "[Unit]\nDescription=Nighterrors Wayland color temperature daemon\nAfter=graphical-session.target\nPartOf=graphical-session.target\n\n[Service]\nType=simple\nExecStart={exec_start}\nRestart=on-failure\nRestartSec=1\nPassEnvironment=WAYLAND_DISPLAY XDG_RUNTIME_DIR\nEnvironment=XDG_RUNTIME_DIR=%t\n\n[Install]\nWantedBy=default.target\n"
    ))
}

fn render_exec_start(options: &ServiceInstallOptions) -> Result<String, String> {
    let binary = current_binary_path()?;
    let mut args = vec![escape_exec_arg(binary.to_string_lossy().as_ref())];
    args.push("run".to_string());
    args.push("--temperature".to_string());
    args.push(options.run_options.temperature_k.to_string());
    args.push("--gamma".to_string());
    args.push(format_float(options.run_options.gamma_pct));
    args.push(format!(
        "--identity={}",
        if options.run_options.identity {
            "true"
        } else {
            "false"
        }
    ));

    if let Some(socket) = &options.socket {
        args.push("--socket".to_string());
        args.push(escape_exec_arg(socket.to_string_lossy().as_ref()));
    }

    for excluded in &options.run_options.excludes {
        args.push("--exclude".to_string());
        args.push(escape_exec_arg(excluded));
    }

    if options.run_options.verbose {
        args.push("--verbose".to_string());
    }

    Ok(args.join(" "))
}

fn current_binary_path() -> Result<PathBuf, String> {
    let path = std::env::current_exe()
        .map_err(|err| format!("failed to resolve current binary path: {err}"))?;
    match fs::canonicalize(&path) {
        Ok(canonical) => Ok(canonical),
        Err(_) => Ok(path),
    }
}

fn escape_exec_arg(value: &str) -> String {
    if value.is_empty() {
        return "\"\"".to_string();
    }

    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || "-_./:@".contains(ch))
    {
        return value.to_string();
    }

    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

fn format_float(value: f64) -> String {
    let candidate = format!("{value:.6}");
    let trimmed = candidate.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::RunOptions;

    fn test_install_options() -> ServiceInstallOptions {
        ServiceInstallOptions {
            socket: Some(PathBuf::from("/tmp/nighterrors.sock")),
            run_options: RunOptions {
                temperature_k: 5500,
                gamma_pct: 95.5,
                identity: false,
                excludes: vec!["eDP-1".to_string(), "HDMI A".to_string()],
                verbose: true,
            },
        }
    }

    #[test]
    fn escape_exec_arg_quotes_unsafe_values() {
        assert_eq!(escape_exec_arg("safe-thing_1"), "safe-thing_1");
        assert_eq!(escape_exec_arg(""), "\"\"");
        assert_eq!(escape_exec_arg("HDMI A"), "\"HDMI A\"");
    }

    #[test]
    fn render_exec_start_includes_run_options() {
        let options = test_install_options();
        let line = render_exec_start(&options).expect("exec start should render");
        assert!(line.contains(" run "));
        assert!(line.contains("--temperature 5500"));
        assert!(line.contains("--gamma 95.5"));
        assert!(line.contains("--identity=false"));
        assert!(line.contains("--socket /tmp/nighterrors.sock"));
        assert!(line.contains("--exclude eDP-1"));
        assert!(line.contains("--exclude \"HDMI A\""));
        assert!(line.contains("--verbose"));
    }

    #[test]
    fn render_unit_file_has_expected_sections() {
        let options = test_install_options();
        let unit = render_unit_file(&options).expect("unit should render");
        assert!(unit.contains("[Unit]"));
        assert!(unit.contains("[Service]"));
        assert!(unit.contains("[Install]"));
        assert!(unit.contains("WantedBy=default.target"));
        assert!(unit.contains("PassEnvironment=WAYLAND_DISPLAY XDG_RUNTIME_DIR"));
    }

    #[test]
    fn user_unit_dir_prefers_xdg_config_home() {
        let temp_dir = std::env::temp_dir().join("nighterrors-xdg-config-home");
        let dir = user_unit_dir_from(temp_dir.to_str(), Some("/home/example"))
            .expect("unit dir should resolve");
        assert_eq!(dir, temp_dir.join("systemd").join("user"));
    }

    #[test]
    fn user_unit_dir_falls_back_to_home() {
        let dir = user_unit_dir_from(None, Some("/home/example")).expect("unit dir should resolve");
        assert_eq!(dir, PathBuf::from("/home/example/.config/systemd/user"));
    }

    #[test]
    fn missing_unit_error_detection() {
        assert!(is_missing_unit_error(
            "Unit nighterrors.service not loaded."
        ));
        assert!(is_missing_unit_error(
            "Unit file nighterrors.service does not exist."
        ));
        assert!(!is_missing_unit_error("permission denied"));
    }
}
