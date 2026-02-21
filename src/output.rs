use crate::cli::{ControlRequest, OutputMode};

pub fn render_response(
    _request: &ControlRequest,
    raw_response: &str,
    mode: OutputMode,
    stdout_is_tty: bool,
) -> String {
    if resolved_raw(mode, stdout_is_tty) {
        return raw_response.to_string();
    }

    if raw_response == "ok" {
        return "Applied.".to_string();
    }

    if let Some(value) = raw_response.strip_prefix("ok temperature=") {
        return format!("Temperature: {value} K");
    }

    if let Some(value) = raw_response.strip_prefix("ok gamma=") {
        return format!("Gamma: {value}%");
    }

    if let Some(value) = raw_response.strip_prefix("ok identity=") {
        let label = match value {
            "true" => "on",
            "false" => "off",
            _ => value,
        };
        return format!("Identity: {label}");
    }

    if let Some(value) = raw_response.strip_prefix("ok backend=") {
        return format!("Backend: {value}");
    }

    if let Some(value) = raw_response.strip_prefix("ok outputs=") {
        return format_outputs(value);
    }

    if let Some(value) = raw_response.strip_prefix("ok excludes=") {
        return format_excludes(value);
    }

    if let Some(value) = raw_response.strip_prefix("ok state=") {
        if let Some(pretty) = format_state(value) {
            return pretty;
        }
    }

    raw_response.to_string()
}

fn resolved_raw(mode: OutputMode, stdout_is_tty: bool) -> bool {
    match mode {
        OutputMode::Raw => true,
        OutputMode::Pretty => false,
        OutputMode::Auto => !stdout_is_tty,
    }
}

fn format_outputs(value: &str) -> String {
    if value == "-" {
        return "Outputs: none".to_string();
    }

    let mut lines = vec!["Outputs:".to_string()];
    for item in split_csv(value) {
        if let Some(name) = item.strip_suffix('*') {
            lines.push(format!("- {name} (excluded)"));
        } else {
            lines.push(format!("- {item}"));
        }
    }

    lines.join("\n")
}

fn format_excludes(value: &str) -> String {
    if value == "-" {
        return "Excluded outputs: none".to_string();
    }

    let values = split_csv(value);
    if values.is_empty() {
        "Excluded outputs: none".to_string()
    } else {
        format!("Excluded outputs: {}", values.join(", "))
    }
}

fn format_state(value: &str) -> Option<String> {
    let mut temperature = None;
    let mut gamma = None;
    let mut identity = None;
    let mut backend = None;
    let mut excludes = None;

    for token in value.split_whitespace() {
        let (key, val) = token.split_once(':')?;
        match key {
            "temperature" => temperature = Some(val),
            "gamma" => gamma = Some(val),
            "identity" => identity = Some(val),
            "backend" => backend = Some(val),
            "excludes" => excludes = Some(val),
            _ => {}
        }
    }

    let temperature = temperature?;
    let gamma = gamma?;
    let identity = identity?;
    let backend = backend?;
    let excludes = excludes?;

    let identity_label = match identity {
        "true" => "on",
        "false" => "off",
        _ => identity,
    };

    let excludes_label = if excludes == "-" {
        "none".to_string()
    } else {
        split_csv(excludes).join(", ")
    };

    Some(format!(
        "State:\nTemperature: {temperature} K\nGamma: {gamma}%\nIdentity: {identity_label}\nBackend: {backend}\nExcluded outputs: {excludes_label}"
    ))
}

fn split_csv(value: &str) -> Vec<&str> {
    value.split(',').filter(|part| !part.is_empty()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{ControlRequest, GetField};

    fn get_state_req() -> ControlRequest {
        ControlRequest::Get(GetField::State)
    }

    #[test]
    fn auto_mode_uses_raw_when_not_tty() {
        let rendered = render_response(&get_state_req(), "ok temperature=6000", OutputMode::Auto, false);
        assert_eq!(rendered, "ok temperature=6000");
    }

    #[test]
    fn auto_mode_uses_pretty_when_tty() {
        let rendered = render_response(&get_state_req(), "ok temperature=6000", OutputMode::Auto, true);
        assert_eq!(rendered, "Temperature: 6000 K");
    }

    #[test]
    fn forced_modes_override_auto() {
        let rendered = render_response(&get_state_req(), "ok gamma=95", OutputMode::Pretty, false);
        assert_eq!(rendered, "Gamma: 95%");

        let rendered = render_response(&get_state_req(), "ok gamma=95", OutputMode::Raw, true);
        assert_eq!(rendered, "ok gamma=95");
    }

    #[test]
    fn unknown_ok_shape_falls_back_to_raw() {
        let rendered = render_response(&get_state_req(), "ok mystery=value", OutputMode::Pretty, true);
        assert_eq!(rendered, "ok mystery=value");
    }

    #[test]
    fn formats_outputs_and_excludes() {
        let rendered = render_response(&get_state_req(), "ok outputs=eDP-1*,HDMI-A-1", OutputMode::Pretty, true);
        assert_eq!(rendered, "Outputs:\n- eDP-1 (excluded)\n- HDMI-A-1");

        let rendered = render_response(&get_state_req(), "ok excludes=-", OutputMode::Pretty, true);
        assert_eq!(rendered, "Excluded outputs: none");
    }

    #[test]
    fn formats_state_payload() {
        let rendered = render_response(
            &get_state_req(),
            "ok state=temperature:6000 gamma:100 identity:false backend:wlr-gamma excludes:eDP-1,@37",
            OutputMode::Pretty,
            true,
        );

        assert_eq!(
            rendered,
            "State:\nTemperature: 6000 K\nGamma: 100%\nIdentity: off\nBackend: wlr-gamma\nExcluded outputs: eDP-1, @37"
        );
    }
}
