use std::ffi::OsString;
use std::path::PathBuf;

pub const DEFAULT_TEMPERATURE_K: u32 = 6000;
pub const DEFAULT_GAMMA_PCT: f64 = 100.0;

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpTopic {
    General,
    Run,
    Set,
    Get,
    Reset,
    Outputs,
    Exclude,
    Stop,
    Status,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Run(RunOptions),
    Control(ControlRequest),
    Help(HelpTopic),
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

const USAGE_GENERAL: &str = "nighterrors v1.2\n\nUsage:\n  nighterrors <command> [options]\n\nCommon Commands:\n  nighterrors run [options]\n  nighterrors set <target> <value> [--socket <PATH>]\n  nighterrors get <field> [--socket <PATH>]\n  nighterrors status [--socket <PATH>] (alias for get state)\n  nighterrors outputs <list|ls> [--socket <PATH>]\n  nighterrors exclude <add|remove|list> ... [--socket <PATH>]\n  nighterrors reset [--socket <PATH>]\n  nighterrors stop [--socket <PATH>]\n\nAliases:\n  run flags: --temp|-t, --g|-g, --id|-i, --ex|-x\n  set targets: temperature|temp|t, gamma|g, identity|id\n  get fields: temperature|temp|t, gamma|g, identity|id, backend|be, state|status\n\nExamples:\n  nighterrors run -t 5500 -g 95 -i off\n  nighterrors set temp +200\n  nighterrors status --raw\n\nControl output modes:\n  --raw      force raw daemon output (ok/error line)\n  --pretty   force human-readable output\n\nMore help:\n  nighterrors help <run|set|get|status|outputs|exclude|reset|stop>";

const USAGE_RUN: &str = "Usage: nighterrors run [options]\n\nOptions:\n  --temperature|--temp|-t <K>      Startup temperature in Kelvin (1000..=20000, default 6000)\n  --gamma|--g|-g <PCT>             Startup gamma percent (0..=200, default 100)\n  --identity|--id|-i [<BOOL>]      Enable identity or set explicitly\n                                    BOOL: true|false|on|off|1|0|yes|no\n  --identity=<BOOL>                Inline value form (also --id=, -i=)\n  --exclude|--ex|-x <OUTPUT_ID>    Exclude an output from filtering (repeatable)\n  --socket <PATH>                  Override control socket path\n  --verbose                        Enable daemon log messages\n  -h, --help                       Show this help\n\nExamples:\n  nighterrors run -t 5500 -g 95 -i off\n  nighterrors run --temp 4800 --id on --exclude eDP-1\n  nighterrors run --identity\n  nighterrors run --identity=false";

const USAGE_SET: &str = "Usage: nighterrors set <target> <value> [--socket <PATH>] [--raw|--pretty]\n\nTargets:\n  temperature|temp|t     <VALUE|+DELTA|-DELTA> in Kelvin\n  gamma|g                <VALUE|+DELTA|-DELTA> in percent\n  identity|id            <true|false|toggle> (aliases: on/off/1/0/yes/no)\n\nExamples:\n  nighterrors set temp 5500\n  nighterrors set g -5\n  nighterrors set id toggle";

const USAGE_GET: &str = "Usage: nighterrors get <field> [--socket <PATH>] [--raw|--pretty]\n\nFields:\n  temperature|temp|t\n  gamma|g\n  identity|id\n  backend|be\n  state|status\n\nExamples:\n  nighterrors get state\n  nighterrors get be --raw";

const USAGE_RESET: &str =
    "Usage: nighterrors reset [--socket <PATH>] [--raw|--pretty]\n\nReset filter state to defaults.";

const USAGE_OUTPUTS: &str = "Usage: nighterrors outputs <list|ls> [--socket <PATH>] [--raw|--pretty]\n\nList detected outputs (`*` means excluded).";

const USAGE_EXCLUDE: &str = "Usage: nighterrors exclude <action> [args] [--socket <PATH>] [--raw|--pretty]\n\nActions:\n  add <OUTPUT_ID>\n  remove|rm|del <OUTPUT_ID>\n  list|ls\n\nExamples:\n  nighterrors exclude add eDP-1\n  nighterrors exclude rm eDP-1\n  nighterrors exclude ls";

const USAGE_STOP: &str =
    "Usage: nighterrors stop [--socket <PATH>] [--raw|--pretty]\n\nStop the running daemon.";

const USAGE_STATUS: &str =
    "Usage: nighterrors status [--socket <PATH>] [--raw|--pretty]\n\nAlias for `nighterrors get state`.";

pub fn usage() -> &'static str {
    USAGE_GENERAL
}

pub fn usage_for(topic: HelpTopic) -> &'static str {
    match topic {
        HelpTopic::General => USAGE_GENERAL,
        HelpTopic::Run => USAGE_RUN,
        HelpTopic::Set => USAGE_SET,
        HelpTopic::Get => USAGE_GET,
        HelpTopic::Reset => USAGE_RESET,
        HelpTopic::Outputs => USAGE_OUTPUTS,
        HelpTopic::Exclude => USAGE_EXCLUDE,
        HelpTopic::Stop => USAGE_STOP,
        HelpTopic::Status => USAGE_STATUS,
    }
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
            command: Command::Help(HelpTopic::General),
        });
    }

    let _program = args.remove(0);
    let output_mode = extract_output_mode_arg(&mut args)?;

    if args.is_empty() {
        return Ok(Cli {
            socket: None,
            output_mode,
            command: Command::Help(HelpTopic::General),
        });
    }

    let command = args.remove(0);

    match command.as_str() {
        "run" => parse_run(args, output_mode),
        "set" => parse_set(args, output_mode),
        "get" => parse_get(args, output_mode),
        "status" => parse_status(args, output_mode),
        "reset" => parse_reset(args, output_mode),
        "outputs" => parse_outputs(args, output_mode),
        "exclude" => parse_exclude(args, output_mode),
        "stop" => parse_stop(args, output_mode),
        "help" => parse_help(args, output_mode),
        "--help" | "-h" => Ok(Cli {
            socket: None,
            output_mode,
            command: Command::Help(HelpTopic::General),
        }),
        "version" | "--version" | "-v" => Ok(Cli {
            socket: None,
            output_mode,
            command: Command::Version,
        }),
        _ => {
            let commands = [
                "run", "set", "get", "status", "reset", "outputs", "exclude", "stop", "help",
                "version",
            ];
            Err(format!(
                "unknown command: {command}{}\n\n{}",
                suggestion_suffix(&command, &commands),
                usage()
            ))
        }
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

fn parse_help(args: Vec<String>, output_mode: OutputMode) -> Result<Cli, String> {
    if args.is_empty() {
        return Ok(Cli {
            socket: None,
            output_mode,
            command: Command::Help(HelpTopic::General),
        });
    }

    if args.len() > 1 {
        return Err("help takes at most one topic".to_string());
    }

    let topic = parse_help_topic(&args[0])?;
    Ok(Cli {
        socket: None,
        output_mode,
        command: Command::Help(topic),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunFlagKind {
    Temperature,
    Gamma,
    Identity,
    Exclude,
    Verbose,
}

fn parse_run(mut args: Vec<String>, output_mode: OutputMode) -> Result<Cli, String> {
    if is_help_only(&args) {
        return Ok(Cli {
            socket: None,
            output_mode,
            command: Command::Help(HelpTopic::Run),
        });
    }

    let socket = extract_socket_arg(&mut args)?;

    let mut options = RunOptions::default();
    let mut i = 0;

    while i < args.len() {
        let token = args[i].as_str();
        let (flag, inline_value) = split_inline_flag_value(token);
        let Some(kind) = normalize_run_flag(flag) else {
            let known_flags = [
                "--temperature",
                "--temp",
                "-t",
                "--gamma",
                "--g",
                "-g",
                "--identity",
                "--id",
                "-i",
                "--exclude",
                "--ex",
                "-x",
                "--verbose",
            ];
            return Err(format!(
                "unknown run flag: {token}{}",
                suggestion_suffix(flag, &known_flags)
            ));
        };

        match kind {
            RunFlagKind::Temperature => {
                let (value, consumed) = take_run_flag_value(flag, inline_value, &args, i)?;
                let parsed = value
                    .parse::<u32>()
                    .map_err(|_| format!("invalid temperature: {value}"))?;
                options.temperature_k = parsed;
                i += consumed;
            }
            RunFlagKind::Gamma => {
                let (value, consumed) = take_run_flag_value(flag, inline_value, &args, i)?;
                let parsed = value
                    .parse::<f64>()
                    .map_err(|_| format!("invalid gamma: {value}"))?;
                options.gamma_pct = parsed;
                i += consumed;
            }
            RunFlagKind::Identity => {
                if let Some(value) = inline_value {
                    options.identity = parse_run_identity_bool(value)?;
                    i += 1;
                    continue;
                }

                if let Some(next) = args.get(i + 1) {
                    if !next.starts_with('-') {
                        options.identity = parse_run_identity_bool(next)?;
                        i += 2;
                        continue;
                    }
                }

                options.identity = true;
                i += 1;
            }
            RunFlagKind::Exclude => {
                let (value, consumed) = take_run_flag_value(flag, inline_value, &args, i)?;
                options.excludes.push(value.to_string());
                i += consumed;
            }
            RunFlagKind::Verbose => {
                if inline_value.is_some() {
                    return Err("run verbose flag does not take a value".to_string());
                }
                options.verbose = true;
                i += 1;
            }
        }
    }

    Ok(Cli {
        socket,
        output_mode,
        command: Command::Run(options),
    })
}

fn normalize_run_flag(value: &str) -> Option<RunFlagKind> {
    match value {
        "--temperature" | "--temp" | "-t" => Some(RunFlagKind::Temperature),
        "--gamma" | "--g" | "-g" => Some(RunFlagKind::Gamma),
        "--identity" | "--id" | "-i" => Some(RunFlagKind::Identity),
        "--exclude" | "--ex" | "-x" => Some(RunFlagKind::Exclude),
        "--verbose" => Some(RunFlagKind::Verbose),
        _ => None,
    }
}

fn split_inline_flag_value(token: &str) -> (&str, Option<&str>) {
    if let Some((flag, value)) = token.split_once('=') {
        (flag, Some(value))
    } else {
        (token, None)
    }
}

fn take_run_flag_value<'a>(
    flag: &str,
    inline_value: Option<&'a str>,
    args: &'a [String],
    index: usize,
) -> Result<(&'a str, usize), String> {
    if let Some(value) = inline_value {
        if value.is_empty() {
            return Err(format!("{flag} requires a value"));
        }
        return Ok((value, 1));
    }

    let value = args
        .get(index + 1)
        .ok_or_else(|| format!("{flag} requires a value"))?;
    Ok((value.as_str(), 2))
}

fn parse_set(mut args: Vec<String>, output_mode: OutputMode) -> Result<Cli, String> {
    if is_help_only(&args) {
        return Ok(Cli {
            socket: None,
            output_mode,
            command: Command::Help(HelpTopic::Set),
        });
    }

    let socket = extract_socket_arg(&mut args)?;

    if args.len() < 2 {
        return Err(format!("set command requires a target and value\n\n{}", usage_for(HelpTopic::Set)));
    }

    let target = args.remove(0);
    let value = args.remove(0);

    if !args.is_empty() {
        return Err(format!("unexpected arguments: {}", args.join(" ")));
    }

    let request = match normalize_set_target(&target) {
        Some("temperature") => ControlRequest::SetTemperature(parse_temperature_change(&value)?),
        Some("gamma") => ControlRequest::SetGamma(parse_gamma_change(&value)?),
        Some("identity") => ControlRequest::SetIdentity(parse_identity_value(&value)?),
        _ => {
            let targets = ["temperature", "temp", "t", "gamma", "g", "identity", "id"];
            return Err(format!(
                "unknown set target: {target}{} (expected temperature|gamma|identity)",
                suggestion_suffix(&target, &targets)
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
    if is_help_only(&args) {
        return Ok(Cli {
            socket: None,
            output_mode,
            command: Command::Help(HelpTopic::Get),
        });
    }

    let socket = extract_socket_arg(&mut args)?;

    if args.len() != 1 {
        return Err(format!("get command requires exactly one field\n\n{}", usage_for(HelpTopic::Get)));
    }

    let field = match normalize_get_field(&args[0]) {
        Some(field) => field,
        None => {
            let fields = [
                "temperature",
                "temp",
                "t",
                "gamma",
                "g",
                "identity",
                "id",
                "backend",
                "be",
                "state",
                "status",
            ];
            return Err(format!(
                "unknown get field: {}{} (expected temperature|gamma|identity|backend|state)",
                args[0],
                suggestion_suffix(&args[0], &fields)
            ));
        }
    };

    Ok(Cli {
        socket,
        output_mode,
        command: Command::Control(ControlRequest::Get(field)),
    })
}

fn parse_status(mut args: Vec<String>, output_mode: OutputMode) -> Result<Cli, String> {
    if is_help_only(&args) {
        return Ok(Cli {
            socket: None,
            output_mode,
            command: Command::Help(HelpTopic::Status),
        });
    }

    let socket = extract_socket_arg(&mut args)?;
    if !args.is_empty() {
        return Err(format!(
            "unexpected arguments for status: {}\n\n{}",
            args.join(" "),
            usage_for(HelpTopic::Status)
        ));
    }

    Ok(Cli {
        socket,
        output_mode,
        command: Command::Control(ControlRequest::Get(GetField::State)),
    })
}

fn parse_reset(mut args: Vec<String>, output_mode: OutputMode) -> Result<Cli, String> {
    if is_help_only(&args) {
        return Ok(Cli {
            socket: None,
            output_mode,
            command: Command::Help(HelpTopic::Reset),
        });
    }

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
    if is_help_only(&args) {
        return Ok(Cli {
            socket: None,
            output_mode,
            command: Command::Help(HelpTopic::Outputs),
        });
    }

    let socket = extract_socket_arg(&mut args)?;

    if args.len() != 1 {
        return Err(format!(
            "outputs command only supports: outputs list\n\n{}",
            usage_for(HelpTopic::Outputs)
        ));
    }

    let action = args.remove(0);
    if action != "list" && action != "ls" {
        return Err(format!(
            "outputs command only supports: outputs list{}",
            suggestion_suffix(&action, &["list", "ls"])
        ));
    }

    Ok(Cli {
        socket,
        output_mode,
        command: Command::Control(ControlRequest::OutputsList),
    })
}

fn parse_exclude(mut args: Vec<String>, output_mode: OutputMode) -> Result<Cli, String> {
    if is_help_only(&args) {
        return Ok(Cli {
            socket: None,
            output_mode,
            command: Command::Help(HelpTopic::Exclude),
        });
    }

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
        "remove" | "rm" | "del" => {
            if args.len() != 1 {
                return Err("exclude remove requires exactly one output id".to_string());
            }
            ControlRequest::ExcludeRemove(args.remove(0))
        }
        "list" | "ls" => {
            if !args.is_empty() {
                return Err("exclude list takes no additional arguments".to_string());
            }
            ControlRequest::ExcludeList
        }
        _ => {
            return Err(format!(
                "exclude command supports only: add <id>, remove <id>, list{}",
                suggestion_suffix(&action, &["add", "remove", "rm", "del", "list", "ls"])
            ));
        }
    };

    Ok(Cli {
        socket,
        output_mode,
        command: Command::Control(request),
    })
}

fn parse_stop(mut args: Vec<String>, output_mode: OutputMode) -> Result<Cli, String> {
    if is_help_only(&args) {
        return Ok(Cli {
            socket: None,
            output_mode,
            command: Command::Help(HelpTopic::Stop),
        });
    }

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

fn parse_help_topic(value: &str) -> Result<HelpTopic, String> {
    match value {
        "help" | "general" => Ok(HelpTopic::General),
        "run" => Ok(HelpTopic::Run),
        "set" => Ok(HelpTopic::Set),
        "get" => Ok(HelpTopic::Get),
        "status" => Ok(HelpTopic::Status),
        "reset" => Ok(HelpTopic::Reset),
        "outputs" => Ok(HelpTopic::Outputs),
        "exclude" => Ok(HelpTopic::Exclude),
        "stop" => Ok(HelpTopic::Stop),
        _ => {
            let topics = [
                "general", "run", "set", "get", "status", "reset", "outputs", "exclude", "stop",
            ];
            Err(format!(
                "unknown help topic: {value}{}",
                suggestion_suffix(value, &topics)
            ))
        }
    }
}

fn normalize_set_target(value: &str) -> Option<&'static str> {
    match value {
        "temperature" | "temp" | "t" => Some("temperature"),
        "gamma" | "g" => Some("gamma"),
        "identity" | "id" => Some("identity"),
        _ => None,
    }
}

fn normalize_get_field(value: &str) -> Option<GetField> {
    match value {
        "temperature" | "temp" | "t" => Some(GetField::Temperature),
        "gamma" | "g" => Some(GetField::Gamma),
        "identity" | "id" => Some(GetField::Identity),
        "backend" | "be" => Some(GetField::Backend),
        "state" | "status" => Some(GetField::State),
        _ => None,
    }
}

fn parse_identity_value(value: &str) -> Result<IdentityValue, String> {
    match value {
        "true" | "on" | "1" | "yes" => Ok(IdentityValue::True),
        "false" | "off" | "0" | "no" => Ok(IdentityValue::False),
        "toggle" => Ok(IdentityValue::Toggle),
        _ => Err("identity value must be one of: true, false, toggle (aliases: on/off/1/0/yes/no)".to_string()),
    }
}

fn parse_run_identity_bool(value: &str) -> Result<bool, String> {
    let parsed = parse_identity_value(value).map_err(|_| {
        "run identity value must be one of: true, false (aliases: on/off/1/0/yes/no)".to_string()
    })?;

    match parsed {
        IdentityValue::True => Ok(true),
        IdentityValue::False => Ok(false),
        IdentityValue::Toggle => Err(
            "run identity value must be one of: true, false (aliases: on/off/1/0/yes/no)"
                .to_string(),
        ),
    }
}

fn is_help_only(args: &[String]) -> bool {
    args.len() == 1 && (args[0] == "--help" || args[0] == "-h")
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

fn suggestion_suffix<'a>(value: &str, candidates: &'a [&'a str]) -> String {
    suggest(value, candidates)
        .map(|candidate| format!(" (did you mean `{candidate}`?)"))
        .unwrap_or_default()
}

fn suggest<'a>(value: &str, candidates: &'a [&'a str]) -> Option<&'a str> {
    let mut best = None;
    let mut best_distance = usize::MAX;

    for candidate in candidates {
        if candidate.eq_ignore_ascii_case(value) {
            return Some(candidate);
        }

        if candidate.starts_with(value) || value.starts_with(candidate) {
            let distance = candidate.len().abs_diff(value.len());
            if distance < best_distance {
                best = Some(*candidate);
                best_distance = distance;
            }
            continue;
        }

        let distance = edit_distance(value, candidate);
        if distance < best_distance {
            best = Some(*candidate);
            best_distance = distance;
        }
    }

    let threshold = if value.len() <= 4 { 1 } else { 2 };
    if best_distance <= threshold { best } else { None }
}

fn edit_distance(a: &str, b: &str) -> usize {
    if a == b {
        return 0;
    }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    if a_chars.is_empty() {
        return b_chars.len();
    }
    if b_chars.is_empty() {
        return a_chars.len();
    }

    let mut prev: Vec<usize> = (0..=b_chars.len()).collect();
    let mut curr = vec![0; b_chars.len() + 1];

    for (i, a_char) in a_chars.iter().enumerate() {
        curr[0] = i + 1;
        for (j, b_char) in b_chars.iter().enumerate() {
            let cost = if a_char == b_char { 0 } else { 1 };
            let insertion = curr[j] + 1;
            let deletion = prev[j + 1] + 1;
            let substitution = prev[j] + cost;
            curr[j + 1] = insertion.min(deletion).min(substitution);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b_chars.len()]
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
        assert_eq!(cli.output_mode, OutputMode::Auto);
    }

    #[test]
    fn parse_run_alias_flags_and_inline_values() {
        let cli = parse_args([
            "nighterrors",
            "run",
            "--temp=5500",
            "--g",
            "95",
            "-x",
            "eDP-1",
            "--ex=HDMI-A-1",
            "--verbose",
        ])
        .expect("parse should succeed");

        match cli.command {
            Command::Run(opts) => {
                assert_eq!(opts.temperature_k, 5500);
                assert!((opts.gamma_pct - 95.0).abs() < f64::EPSILON);
                assert_eq!(
                    opts.excludes,
                    vec!["eDP-1".to_string(), "HDMI-A-1".to_string()]
                );
                assert!(opts.verbose);
            }
            _ => panic!("expected run command"),
        }
    }

    #[test]
    fn parse_run_identity_forms() {
        let cli = parse_args(["nighterrors", "run", "--identity"]).expect("parse should succeed");
        match cli.command {
            Command::Run(opts) => assert!(opts.identity),
            _ => panic!("expected run command"),
        }

        let cli = parse_args(["nighterrors", "run", "--id", "on"]).expect("parse should succeed");
        match cli.command {
            Command::Run(opts) => assert!(opts.identity),
            _ => panic!("expected run command"),
        }

        let cli = parse_args(["nighterrors", "run", "-i", "off"]).expect("parse should succeed");
        match cli.command {
            Command::Run(opts) => assert!(!opts.identity),
            _ => panic!("expected run command"),
        }

        let cli =
            parse_args(["nighterrors", "run", "--identity=false"]).expect("parse should succeed");
        match cli.command {
            Command::Run(opts) => assert!(!opts.identity),
            _ => panic!("expected run command"),
        }
    }

    #[test]
    fn parse_run_identity_invalid_value_fails() {
        let err = parse_args(["nighterrors", "run", "--id", "maybe"]).expect_err("must fail");
        assert!(err.contains("run identity value must be one of"));
    }

    #[test]
    fn parse_run_unknown_flag_suggests_nearest() {
        let err = parse_args(["nighterrors", "run", "--temprature", "5500"]).expect_err("must fail");
        assert!(err.contains("unknown run flag"));
        assert!(err.contains("did you mean"));
    }

    #[test]
    fn help_text_includes_new_sections_and_examples() {
        let general = usage_for(HelpTopic::General);
        assert!(general.contains("Common Commands"));
        assert!(general.contains("Aliases"));
        assert!(general.contains("Examples"));

        let run = usage_for(HelpTopic::Run);
        assert!(run.contains("--temperature|--temp|-t"));
        assert!(run.contains("BOOL: true|false|on|off|1|0|yes|no"));
        assert!(run.contains("nighterrors run -t 5500 -g 95 -i off"));
    }

    #[test]
    fn parse_set_temperature_alias() {
        let cli = parse_args(["nighterrors", "set", "temp", "+250"]).expect("parse should succeed");
        match cli.command {
            Command::Control(ControlRequest::SetTemperature(TemperatureChange::Relative(v))) => {
                assert_eq!(v, 250)
            }
            _ => panic!("expected relative temperature set"),
        }
    }

    #[test]
    fn parse_set_gamma_alias() {
        let cli = parse_args(["nighterrors", "set", "g", "95.5"]).expect("parse");
        match cli.command {
            Command::Control(ControlRequest::SetGamma(GammaChange::Absolute(v))) => {
                assert!((v - 95.5).abs() < f64::EPSILON);
            }
            _ => panic!("expected absolute gamma set"),
        }
    }

    #[test]
    fn parse_identity_alias_values() {
        let cli = parse_args(["nighterrors", "set", "id", "on"]).expect("parse");
        match cli.command {
            Command::Control(ControlRequest::SetIdentity(IdentityValue::True)) => {}
            _ => panic!("expected identity=true"),
        }

        let cli = parse_args(["nighterrors", "set", "identity", "toggle"]).expect("parse");
        match cli.command {
            Command::Control(ControlRequest::SetIdentity(IdentityValue::Toggle)) => {}
            _ => panic!("expected identity toggle"),
        }
    }

    #[test]
    fn status_alias_maps_to_get_state() {
        let cli = parse_args(["nighterrors", "status"]).expect("parse");
        assert_eq!(
            cli.command,
            Command::Control(ControlRequest::Get(GetField::State))
        );
    }

    #[test]
    fn parse_get_backend_alias() {
        let cli = parse_args(["nighterrors", "get", "be"]).expect("parse");
        assert_eq!(
            cli.command,
            Command::Control(ControlRequest::Get(GetField::Backend))
        );
    }

    #[test]
    fn parse_outputs_ls_alias() {
        let cli = parse_args(["nighterrors", "outputs", "ls"]).expect("parse");
        assert_eq!(cli.command, Command::Control(ControlRequest::OutputsList));
    }

    #[test]
    fn parse_exclude_remove_aliases() {
        let cli = parse_args(["nighterrors", "exclude", "rm", "eDP-1"]).expect("parse");
        assert_eq!(
            cli.command,
            Command::Control(ControlRequest::ExcludeRemove("eDP-1".to_string()))
        );

        let cli = parse_args(["nighterrors", "exclude", "del", "eDP-1"]).expect("parse");
        assert_eq!(
            cli.command,
            Command::Control(ControlRequest::ExcludeRemove("eDP-1".to_string()))
        );
    }

    #[test]
    fn parse_output_mode_flags_anywhere() {
        let cli = parse_args(["nighterrors", "--raw", "get", "temperature"]).expect("parse");
        assert_eq!(cli.output_mode, OutputMode::Raw);

        let cli = parse_args(["nighterrors", "get", "temperature", "--pretty"]).expect("parse");
        assert_eq!(cli.output_mode, OutputMode::Pretty);
    }

    #[test]
    fn parse_output_mode_conflict_fails() {
        let err = parse_args(["nighterrors", "--raw", "get", "state", "--pretty"]).expect_err("must fail");
        assert!(err.contains("cannot be used together"));
    }

    #[test]
    fn parse_help_topic_and_command_help() {
        let cli = parse_args(["nighterrors", "help", "set"]).expect("parse");
        assert_eq!(cli.command, Command::Help(HelpTopic::Set));

        let cli = parse_args(["nighterrors", "run", "--help"]).expect("parse");
        assert_eq!(cli.command, Command::Help(HelpTopic::Run));

        let cli = parse_args(["nighterrors", "set", "--help"]).expect("parse");
        assert_eq!(cli.command, Command::Help(HelpTopic::Set));

        let cli = parse_args(["nighterrors", "status", "--help"]).expect("parse");
        assert_eq!(cli.command, Command::Help(HelpTopic::Status));
    }

    #[test]
    fn roundtrip_wire() {
        let req = ControlRequest::ExcludeAdd("eDP-1".to_string());
        let wire = req.to_wire();
        let parsed = ControlRequest::from_wire(&wire).expect("wire parse should succeed");
        assert_eq!(req, parsed);
    }

    #[test]
    fn parse_unknown_command_fails_with_suggestion() {
        let err = parse_args(["nighterrors", "statsu"]).expect_err("must fail");
        assert!(err.contains("unknown command"));
        assert!(err.contains("did you mean"));
    }

    #[test]
    fn edit_distance_smoke() {
        assert_eq!(edit_distance("status", "status"), 0);
        assert_eq!(edit_distance("status", "statuz"), 1);
        assert!(edit_distance("abc", "xyz") >= 2);
    }
}
