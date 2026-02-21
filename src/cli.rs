use std::ffi::OsString;
use std::path::PathBuf;

pub const DEFAULT_TEMPERATURE_K: u32 = 6000;
pub const DEFAULT_GAMMA_PCT: f64 = 100.0;

#[derive(Debug, Clone)]
pub struct RunOptions {
    pub temperature_k: u32,
    pub gamma_pct: f64,
    pub identity: bool,
    pub excludes: Vec<String>,
    pub verbose: bool,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            temperature_k: DEFAULT_TEMPERATURE_K,
            gamma_pct: DEFAULT_GAMMA_PCT,
            identity: false,
            excludes: Vec::new(),
            verbose: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TemperatureChange {
    Absolute(i64),
    Relative(i64),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GammaChange {
    Absolute(f64),
    Relative(f64),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IdentityValue {
    True,
    False,
    Toggle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GetField {
    Temperature,
    Gamma,
    Identity,
    Backend,
    State,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlRequest {
    SetTemperature(TemperatureChange),
    SetGamma(GammaChange),
    SetIdentity(IdentityValue),
    Get(GetField),
    Reset,
    OutputsList,
    ExcludeAdd(String),
    ExcludeRemove(String),
    ExcludeList,
    Stop,
}

#[derive(Debug, Clone)]
pub enum Command {
    Run(RunOptions),
    Control(ControlRequest),
    Help,
    Version,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Auto,
    Raw,
    Pretty,
}

#[derive(Debug, Clone)]
pub struct Cli {
    pub socket: Option<PathBuf>,
    pub output_mode: OutputMode,
    pub command: Command,
}

pub fn usage() -> &'static str {
    "nighterrors v1\n\nCommands:\n  nighterrors run [--temperature <K>] [--gamma <PCT>] [--identity] [--exclude <OUTPUT_ID>]... [--socket <PATH>] [--verbose]\n  nighterrors set temperature <VALUE|+DELTA|-DELTA> [--socket <PATH>]\n  nighterrors set gamma <VALUE|+DELTA|-DELTA> [--socket <PATH>]\n  nighterrors set identity <true|false|toggle> [--socket <PATH>]\n  nighterrors get <temperature|gamma|identity|backend|state> [--socket <PATH>]\n  nighterrors reset [--socket <PATH>]\n  nighterrors outputs list [--socket <PATH>]\n  nighterrors exclude add <OUTPUT_ID> [--socket <PATH>]\n  nighterrors exclude remove <OUTPUT_ID> [--socket <PATH>]\n  nighterrors exclude list [--socket <PATH>]\n  nighterrors stop [--socket <PATH>]"
}

pub fn parse_args<I, S>(args: I) -> Result<Cli, String>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let mut args: Vec<String> = args
        .into_iter()
        .map(|s| s.into().to_string_lossy().into_owned())
        .collect();

    if args.is_empty() {
        return Ok(Cli {
            socket: None,
            output_mode: OutputMode::Auto,
            command: Command::Help,
        });
    }

    let _program = args.remove(0);

    let output_mode = extract_output_mode_arg(&mut args)?;

    if args.is_empty() {
        return Ok(Cli {
            socket: None,
            output_mode,
            command: Command::Help,
        });
    }

    let command = args.remove(0);

    match command.as_str() {
        "run" => parse_run(args, output_mode),
        "set" => parse_set(args, output_mode),
        "get" => parse_get(args, output_mode),
        "reset" => parse_reset(args, output_mode),
        "outputs" => parse_outputs(args, output_mode),
        "exclude" => parse_exclude(args, output_mode),
        "stop" => parse_stop(args, output_mode),
        "help" | "--help" | "-h" => Ok(Cli {
            socket: None,
            output_mode,
            command: Command::Help,
        }),
        "version" | "--version" | "-v" => Ok(Cli {
            socket: None,
            output_mode,
            command: Command::Version,
        }),
        _ => Err(format!("unknown command: {command}\n\n{}", usage())),
    }
}

impl ControlRequest {
    pub fn to_wire(&self) -> String {
        match self {
            ControlRequest::SetTemperature(value) => match value {
                TemperatureChange::Absolute(v) => format!("set temperature {v}"),
                TemperatureChange::Relative(v) => format!("set temperature {:+}", v),
            },
            ControlRequest::SetGamma(value) => match value {
                GammaChange::Absolute(v) => format!("set gamma {}", format_float(*v)),
                GammaChange::Relative(v) => format!("set gamma {:+}", *v),
            },
            ControlRequest::SetIdentity(value) => match value {
                IdentityValue::True => "set identity true".to_string(),
                IdentityValue::False => "set identity false".to_string(),
                IdentityValue::Toggle => "set identity toggle".to_string(),
            },
            ControlRequest::Get(field) => match field {
                GetField::Temperature => "get temperature".to_string(),
                GetField::Gamma => "get gamma".to_string(),
                GetField::Identity => "get identity".to_string(),
                GetField::Backend => "get backend".to_string(),
                GetField::State => "get state".to_string(),
            },
            ControlRequest::Reset => "reset".to_string(),
            ControlRequest::OutputsList => "outputs list".to_string(),
            ControlRequest::ExcludeAdd(value) => format!("exclude add {value}"),
            ControlRequest::ExcludeRemove(value) => format!("exclude remove {value}"),
            ControlRequest::ExcludeList => "exclude list".to_string(),
            ControlRequest::Stop => "stop".to_string(),
        }
    }

    pub fn from_wire(line: &str) -> Result<Self, String> {
        let mut args = vec!["nighterrors".to_string()];
        args.extend(line.split_whitespace().map(ToOwned::to_owned));

        let parsed = parse_args(args)?;

        match parsed.command {
            Command::Control(request) => Ok(request),
            _ => Err("control request expected".to_string()),
        }
    }
}

fn parse_run(mut args: Vec<String>, output_mode: OutputMode) -> Result<Cli, String> {
    let socket = extract_socket_arg(&mut args)?;

    let mut options = RunOptions::default();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--temperature" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--temperature requires a value".to_string())?;
                let parsed = value
                    .parse::<u32>()
                    .map_err(|_| format!("invalid temperature: {value}"))?;
                options.temperature_k = parsed;
                i += 2;
            }
            "--gamma" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--gamma requires a value".to_string())?;
                let parsed = value
                    .parse::<f64>()
                    .map_err(|_| format!("invalid gamma: {value}"))?;
                options.gamma_pct = parsed;
                i += 2;
            }
            "--identity" => {
                options.identity = true;
                i += 1;
            }
            "--exclude" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--exclude requires an output id".to_string())?;
                options.excludes.push(value.clone());
                i += 2;
            }
            "--verbose" => {
                options.verbose = true;
                i += 1;
            }
            unknown => {
                return Err(format!("unknown run flag: {unknown}"));
            }
        }
    }

    Ok(Cli {
        socket,
        output_mode,
        command: Command::Run(options),
    })
}

fn parse_set(mut args: Vec<String>, output_mode: OutputMode) -> Result<Cli, String> {
    let socket = extract_socket_arg(&mut args)?;

    if args.len() < 2 {
        return Err("set command requires a target and value".to_string());
    }

    let target = args.remove(0);
    let value = args.remove(0);

    if !args.is_empty() {
        return Err(format!("unexpected arguments: {}", args.join(" ")));
    }

    let request = match target.as_str() {
        "temperature" => ControlRequest::SetTemperature(parse_temperature_change(&value)?),
        "gamma" => ControlRequest::SetGamma(parse_gamma_change(&value)?),
        "identity" => {
            let parsed = match value.as_str() {
                "true" => IdentityValue::True,
                "false" => IdentityValue::False,
                "toggle" => IdentityValue::Toggle,
                _ => {
                    return Err("identity value must be one of: true, false, toggle".to_string());
                }
            };
            ControlRequest::SetIdentity(parsed)
        }
        _ => {
            return Err(format!(
                "unknown set target: {target} (expected temperature|gamma|identity)"
            ));
        }
    };

    Ok(Cli {
        socket,
        output_mode,
        command: Command::Control(request),
    })
}

fn parse_get(mut args: Vec<String>, output_mode: OutputMode) -> Result<Cli, String> {
    let socket = extract_socket_arg(&mut args)?;

    if args.len() != 1 {
        return Err("get command requires exactly one field".to_string());
    }

    let field = match args[0].as_str() {
        "temperature" => GetField::Temperature,
        "gamma" => GetField::Gamma,
        "identity" => GetField::Identity,
        "backend" => GetField::Backend,
        "state" => GetField::State,
        _ => {
            return Err(format!(
                "unknown get field: {} (expected temperature|gamma|identity|backend|state)",
                args[0]
            ));
        }
    };

    Ok(Cli {
        socket,
        output_mode,
        command: Command::Control(ControlRequest::Get(field)),
    })
}

fn parse_reset(mut args: Vec<String>, output_mode: OutputMode) -> Result<Cli, String> {
    let socket = extract_socket_arg(&mut args)?;
    if !args.is_empty() {
        return Err(format!(
            "unexpected arguments for reset: {}",
            args.join(" ")
        ));
    }

    Ok(Cli {
        socket,
        output_mode,
        command: Command::Control(ControlRequest::Reset),
    })
}

fn parse_outputs(mut args: Vec<String>, output_mode: OutputMode) -> Result<Cli, String> {
    let socket = extract_socket_arg(&mut args)?;

    if args.len() != 1 || args[0] != "list" {
        return Err("outputs command only supports: outputs list".to_string());
    }

    Ok(Cli {
        socket,
        output_mode,
        command: Command::Control(ControlRequest::OutputsList),
    })
}

fn parse_exclude(mut args: Vec<String>, output_mode: OutputMode) -> Result<Cli, String> {
    let socket = extract_socket_arg(&mut args)?;

    if args.is_empty() {
        return Err("exclude command requires an action".to_string());
    }

    let action = args.remove(0);

    let request = match action.as_str() {
        "add" => {
            if args.len() != 1 {
                return Err("exclude add requires exactly one output id".to_string());
            }
            ControlRequest::ExcludeAdd(args.remove(0))
        }
        "remove" => {
            if args.len() != 1 {
                return Err("exclude remove requires exactly one output id".to_string());
            }
            ControlRequest::ExcludeRemove(args.remove(0))
        }
        "list" => {
            if !args.is_empty() {
                return Err("exclude list takes no additional arguments".to_string());
            }
            ControlRequest::ExcludeList
        }
        _ => {
            return Err("exclude command supports only: add <id>, remove <id>, list".to_string());
        }
    };

    Ok(Cli {
        socket,
        output_mode,
        command: Command::Control(request),
    })
}

fn parse_stop(mut args: Vec<String>, output_mode: OutputMode) -> Result<Cli, String> {
    let socket = extract_socket_arg(&mut args)?;
    if !args.is_empty() {
        return Err(format!("unexpected arguments for stop: {}", args.join(" ")));
    }

    Ok(Cli {
        socket,
        output_mode,
        command: Command::Control(ControlRequest::Stop),
    })
}

fn extract_socket_arg(args: &mut Vec<String>) -> Result<Option<PathBuf>, String> {
    let mut socket = None;
    let mut i = 0;

    while i < args.len() {
        if args[i] == "--socket" {
            if socket.is_some() {
                return Err("--socket specified more than once".to_string());
            }
            let path = args
                .get(i + 1)
                .ok_or_else(|| "--socket requires a path".to_string())?
                .clone();

            args.drain(i..=i + 1);
            socket = Some(PathBuf::from(path));
            continue;
        }

        i += 1;
    }

    Ok(socket)
}

fn extract_output_mode_arg(args: &mut Vec<String>) -> Result<OutputMode, String> {
    let mut mode = OutputMode::Auto;
    let mut i = 0;

    while i < args.len() {
        let next_mode = match args[i].as_str() {
            "--raw" => Some(OutputMode::Raw),
            "--pretty" => Some(OutputMode::Pretty),
            _ => None,
        };

        let Some(next_mode) = next_mode else {
            i += 1;
            continue;
        };

        if mode != OutputMode::Auto && mode != next_mode {
            return Err("--raw and --pretty cannot be used together".to_string());
        }

        mode = next_mode;
        args.remove(i);
    }

    Ok(mode)
}

fn parse_temperature_change(value: &str) -> Result<TemperatureChange, String> {
    if let Some(rest) = value.strip_prefix('+') {
        let parsed = rest
            .parse::<i64>()
            .map_err(|_| format!("invalid temperature delta: {value}"))?;
        return Ok(TemperatureChange::Relative(parsed));
    }

    if let Some(rest) = value.strip_prefix('-') {
        let parsed = rest
            .parse::<i64>()
            .map_err(|_| format!("invalid temperature delta: {value}"))?;
        return Ok(TemperatureChange::Relative(-parsed));
    }

    let parsed = value
        .parse::<i64>()
        .map_err(|_| format!("invalid temperature value: {value}"))?;

    Ok(TemperatureChange::Absolute(parsed))
}

fn parse_gamma_change(value: &str) -> Result<GammaChange, String> {
    if let Some(rest) = value.strip_prefix('+') {
        let parsed = rest
            .parse::<f64>()
            .map_err(|_| format!("invalid gamma delta: {value}"))?;
        return Ok(GammaChange::Relative(parsed));
    }

    if let Some(rest) = value.strip_prefix('-') {
        let parsed = rest
            .parse::<f64>()
            .map_err(|_| format!("invalid gamma delta: {value}"))?;
        return Ok(GammaChange::Relative(-parsed));
    }

    let parsed = value
        .parse::<f64>()
        .map_err(|_| format!("invalid gamma value: {value}"))?;

    Ok(GammaChange::Absolute(parsed))
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

    #[test]
    fn parse_run_defaults() {
        let cli = parse_args(["nighterrors", "run"]).expect("parse should succeed");
        match cli.command {
            Command::Run(opts) => {
                assert_eq!(opts.temperature_k, DEFAULT_TEMPERATURE_K);
                assert_eq!(opts.gamma_pct, DEFAULT_GAMMA_PCT);
                assert!(!opts.identity);
                assert!(opts.excludes.is_empty());
            }
            _ => panic!("expected run command"),
        }
    }

    #[test]
    fn parse_set_temperature_relative() {
        let cli = parse_args(["nighterrors", "set", "temperature", "+250"])
            .expect("parse should succeed");
        match cli.command {
            Command::Control(ControlRequest::SetTemperature(TemperatureChange::Relative(v))) => {
                assert_eq!(v, 250)
            }
            _ => panic!("expected relative temperature set"),
        }
    }

    #[test]
    fn parse_set_gamma_absolute() {
        let cli = parse_args(["nighterrors", "set", "gamma", "95.5"]).expect("parse");
        match cli.command {
            Command::Control(ControlRequest::SetGamma(GammaChange::Absolute(v))) => {
                assert!((v - 95.5).abs() < f64::EPSILON);
            }
            _ => panic!("expected absolute gamma set"),
        }
    }

    #[test]
    fn parse_identity_toggle() {
        let cli = parse_args(["nighterrors", "set", "identity", "toggle"]).expect("parse");
        match cli.command {
            Command::Control(ControlRequest::SetIdentity(IdentityValue::Toggle)) => {}
            _ => panic!("expected identity toggle"),
        }
    }

    #[test]
    fn roundtrip_wire() {
        let req = ControlRequest::ExcludeAdd("eDP-1".to_string());
        let wire = req.to_wire();
        let parsed = ControlRequest::from_wire(&wire).expect("wire parse should succeed");
        assert_eq!(req, parsed);
    }

    #[test]
    fn parse_unknown_command_fails() {
        let err = parse_args(["nighterrors", "explode"]).expect_err("must fail");
        assert!(err.contains("unknown command"));
    }
}
