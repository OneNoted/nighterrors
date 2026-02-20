use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::os::fd::{AsFd, AsRawFd};
use std::path::PathBuf;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

use wayland_client::protocol::{wl_output, wl_registry};
use wayland_client::{Connection, Dispatch, EventQueue, Proxy, QueueHandle};

use crate::backends::hyprland_ctm;
use crate::backends::wlr_gamma::{self, WlrControlState};
use crate::cli::{
    ControlRequest, GammaChange, GetField, IdentityValue, RunOptions, TemperatureChange,
    DEFAULT_GAMMA_PCT, DEFAULT_TEMPERATURE_K,
};
use crate::color;
use crate::ipc::{self, IpcRequest};
use crate::protocols::hyprland_ctm::hyprland_ctm_control_manager_v1;
use crate::protocols::wlr_gamma::{zwlr_gamma_control_manager_v1, zwlr_gamma_control_v1};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    HyprlandCtm,
    WlrGamma,
}

impl fmt::Display for BackendKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackendKind::HyprlandCtm => write!(f, "hyprland-ctm"),
            BackendKind::WlrGamma => write!(f, "wlr-gamma"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FilterState {
    pub temperature_k: u32,
    pub gamma_pct: f64,
    pub identity: bool,
    pub excluded_ids: BTreeSet<String>,
}

impl FilterState {
    fn from_run_options(options: &RunOptions) -> Self {
        Self {
            temperature_k: options.temperature_k,
            gamma_pct: options.gamma_pct,
            identity: options.identity,
            excluded_ids: options.excludes.iter().cloned().collect(),
        }
    }

    fn reset_to_defaults(&mut self) {
        self.temperature_k = DEFAULT_TEMPERATURE_K;
        self.gamma_pct = DEFAULT_GAMMA_PCT;
        self.identity = false;
    }
}

#[derive(Debug, Clone)]
pub struct OutputInfo {
    pub global_id: u32,
    pub wl_output: wl_output::WlOutput,
    pub name_opt: Option<String>,
    pub description_opt: Option<String>,
}

impl OutputInfo {
    fn new(global_id: u32, wl_output: wl_output::WlOutput) -> Self {
        Self {
            global_id,
            wl_output,
            name_opt: None,
            description_opt: None,
        }
    }

    fn primary_id(&self) -> String {
        self.name_opt
            .clone()
            .unwrap_or_else(|| format!("@{}", self.global_id))
    }

    fn is_excluded(&self, excluded_ids: &BTreeSet<String>) -> bool {
        excluded_ids.contains(&format!("@{}", self.global_id))
            || self
                .name_opt
                .as_ref()
                .map(|name| excluded_ids.contains(name))
                .unwrap_or(false)
    }
}

#[derive(Debug)]
struct WaylandState {
    verbose: bool,
    outputs: BTreeMap<u32, OutputInfo>,
    hyprland_manager: Option<hyprland_ctm_control_manager_v1::HyprlandCtmControlManagerV1>,
    hyprland_blocked: bool,
    wlr_manager: Option<zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1>,
    wlr_controls: BTreeMap<u32, WlrControlState>,
    selected_backend: Option<BackendKind>,
    pending_reapply: bool,
}

impl WaylandState {
    fn new(verbose: bool) -> Self {
        Self {
            verbose,
            outputs: BTreeMap::new(),
            hyprland_manager: None,
            hyprland_blocked: false,
            wlr_manager: None,
            wlr_controls: BTreeMap::new(),
            selected_backend: None,
            pending_reapply: false,
        }
    }

    fn ensure_wlr_controls(&mut self, qh: &QueueHandle<Self>) {
        if self.selected_backend != Some(BackendKind::WlrGamma) {
            return;
        }

        let Some(manager) = self.wlr_manager.clone() else {
            return;
        };

        let missing: Vec<u32> = self
            .outputs
            .keys()
            .copied()
            .filter(|global_id| !self.wlr_controls.contains_key(global_id))
            .collect();

        for global_id in missing {
            if let Some(output) = self.outputs.get(&global_id) {
                let control = wlr_gamma::create_control(&manager, &output.wl_output, qh, global_id);
                self.wlr_controls.insert(global_id, control);
            }
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for WaylandState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } => {
                if interface == wl_output::WlOutput::interface().name {
                    let bind_version = version.min(4);
                    let output =
                        registry.bind::<wl_output::WlOutput, _, _>(name, bind_version, qh, name);

                    state.outputs.insert(name, OutputInfo::new(name, output));
                    state.ensure_wlr_controls(qh);
                    state.pending_reapply = true;
                    vlog(state.verbose, &format!("bound wl_output global {name}"));
                } else if interface
                    == hyprland_ctm_control_manager_v1::HyprlandCtmControlManagerV1::interface()
                        .name
                {
                    let bind_version = version.min(2);
                    let manager = registry
                        .bind::<hyprland_ctm_control_manager_v1::HyprlandCtmControlManagerV1, _, _>(
                        name,
                        bind_version,
                        qh,
                        (),
                    );
                    state.hyprland_manager = Some(manager);
                    vlog(
                        state.verbose,
                        &format!(
                            "found hyprland-ctm-control manager global {name} (v{})",
                            bind_version
                        ),
                    );
                } else if interface
                    == zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1::interface().name
                {
                    let manager = registry
                        .bind::<zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1, _, _>(
                            name,
                            1,
                            qh,
                            (),
                        );
                    state.wlr_manager = Some(manager);
                    vlog(
                        state.verbose,
                        &format!("found wlr gamma-control manager global {name}"),
                    );
                }
            }
            wl_registry::Event::GlobalRemove { name } => {
                if state.outputs.remove(&name).is_some() {
                    state.pending_reapply = true;
                    vlog(state.verbose, &format!("output global {name} removed"));
                }

                if let Some(control) = state.wlr_controls.remove(&name) {
                    wlr_gamma::destroy_control(&control);
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_output::WlOutput, u32> for WaylandState {
    fn event(
        state: &mut Self,
        _proxy: &wl_output::WlOutput,
        event: wl_output::Event,
        global_id: &u32,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        let Some(output) = state.outputs.get_mut(global_id) else {
            return;
        };

        match event {
            wl_output::Event::Name { name } => {
                if output.name_opt.as_deref() != Some(name.as_str()) {
                    output.name_opt = Some(name);
                    state.pending_reapply = true;
                }
            }
            wl_output::Event::Description { description } => {
                output.description_opt = Some(description);
            }
            _ => {}
        }
    }
}

impl Dispatch<hyprland_ctm_control_manager_v1::HyprlandCtmControlManagerV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _proxy: &hyprland_ctm_control_manager_v1::HyprlandCtmControlManagerV1,
        event: hyprland_ctm_control_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            hyprland_ctm_control_manager_v1::Event::Blocked => {
                state.hyprland_blocked = true;
            }
        }
    }
}

impl Dispatch<zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1,
        _event: zwlr_gamma_control_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwlr_gamma_control_v1::ZwlrGammaControlV1, u32> for WaylandState {
    fn event(
        state: &mut Self,
        _proxy: &zwlr_gamma_control_v1::ZwlrGammaControlV1,
        event: zwlr_gamma_control_v1::Event,
        global_id: &u32,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_gamma_control_v1::Event::GammaSize { size } => {
                if let Some(control) = state.wlr_controls.get_mut(global_id) {
                    control.gamma_size = Some(size);
                    control.failed = false;
                    state.pending_reapply = true;
                }
            }
            zwlr_gamma_control_v1::Event::Failed => {
                if let Some(old) = state.wlr_controls.remove(global_id) {
                    wlr_gamma::destroy_control(&old);
                }

                if let (Some(manager), Some(output)) =
                    (state.wlr_manager.clone(), state.outputs.get(global_id))
                {
                    let recreated =
                        wlr_gamma::create_control(&manager, &output.wl_output, qh, *global_id);
                    state.wlr_controls.insert(*global_id, recreated);
                }

                state.pending_reapply = true;
            }
        }
    }
}

pub fn run(options: RunOptions, socket_override: Option<PathBuf>) -> Result<(), String> {
    validate_temperature(options.temperature_k)?;
    validate_gamma(options.gamma_pct)?;

    let socket_path = socket_override.unwrap_or_else(ipc::default_socket_path);

    let (request_tx, request_rx) = mpsc::channel::<IpcRequest>();
    let ipc_server = ipc::start_server(&socket_path, request_tx)?;

    let connection =
        Connection::connect_to_env().map_err(|e| format!("failed to connect to Wayland: {e}"))?;

    let mut event_queue: EventQueue<WaylandState> = connection.new_event_queue();
    let qh = event_queue.handle();
    let display = connection.display();

    let _registry = display.get_registry(&qh, ());

    let mut wl_state = WaylandState::new(options.verbose);

    event_queue
        .roundtrip(&mut wl_state)
        .map_err(|e| format!("wayland roundtrip failed: {e}"))?;
    event_queue
        .roundtrip(&mut wl_state)
        .map_err(|e| format!("wayland roundtrip failed: {e}"))?;

    let backend = select_backend(&wl_state)?;

    if backend == BackendKind::HyprlandCtm && wl_state.hyprland_blocked {
        return Err("hyprland CTM manager is blocked by another client".to_string());
    }

    wl_state.selected_backend = Some(backend);
    wl_state.ensure_wlr_controls(&qh);

    if backend == BackendKind::WlrGamma {
        event_queue
            .roundtrip(&mut wl_state)
            .map_err(|e| format!("wayland roundtrip failed: {e}"))?;
    }

    vlog(
        wl_state.verbose,
        &format!(
            "daemon started with backend {}, socket {}",
            backend,
            socket_path.display()
        ),
    );

    let mut filter_state = FilterState::from_run_options(&options);
    wl_state.pending_reapply = true;

    let mut should_stop = false;

    while !should_stop {
        event_queue
            .dispatch_pending(&mut wl_state)
            .map_err(|e| format!("wayland dispatch failed: {e}"))?;

        if wl_state.pending_reapply {
            if let Err(err) = apply_filter(&mut wl_state, &filter_state, backend) {
                vlog(wl_state.verbose, &format!("apply failed: {err}"));
            }
            wl_state.pending_reapply = false;
        }

        loop {
            match request_rx.try_recv() {
                Ok(msg) => {
                    let request_result =
                        handle_request(&msg.line, &mut filter_state, &mut wl_state, backend);
                    let mut response = request_result.response;

                    if request_result.needs_apply {
                        event_queue
                            .dispatch_pending(&mut wl_state)
                            .map_err(|e| format!("wayland dispatch failed: {e}"))?;

                        match apply_filter(&mut wl_state, &filter_state, backend) {
                            Ok(()) => {}
                            Err(err) => response = format!("error: apply failed: {err}"),
                        }
                    }

                    let _ = msg.reply_tx.send(response);
                    if request_result.should_stop {
                        should_stop = true;
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    should_stop = true;
                    break;
                }
            }
        }

        if should_stop {
            break;
        }

        if wl_state.pending_reapply {
            continue;
        }

        event_queue
            .flush()
            .map_err(|e| format!("wayland flush failed: {e}"))?;

        if let Some(read_guard) = event_queue.prepare_read() {
            let fd = event_queue.as_fd().as_raw_fd();
            let mut poll_fd = libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            };

            let poll_result = unsafe { libc::poll(&mut poll_fd, 1, 50) };

            if poll_result < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() != std::io::ErrorKind::Interrupted {
                    return Err(format!("poll failed: {err}"));
                }
                drop(read_guard);
            } else if poll_result > 0 && (poll_fd.revents & libc::POLLIN) != 0 {
                read_guard
                    .read()
                    .map_err(|e| format!("wayland read failed: {e}"))?;
            } else {
                drop(read_guard);
            }
        } else {
            thread::sleep(Duration::from_millis(10));
        }
    }

    ipc_server.shutdown();
    Ok(())
}

fn select_backend(wl_state: &WaylandState) -> Result<BackendKind, String> {
    if wl_state.hyprland_manager.is_some() {
        return Ok(BackendKind::HyprlandCtm);
    }

    if wl_state.wlr_manager.is_some() {
        return Ok(BackendKind::WlrGamma);
    }

    Err(
        "no supported color-control protocol found (need hyprland-ctm-control-v1 or zwlr_gamma_control_manager_v1)"
            .to_string(),
    )
}

fn apply_filter(
    wl_state: &mut WaylandState,
    filter: &FilterState,
    backend: BackendKind,
) -> Result<(), String> {
    match backend {
        BackendKind::HyprlandCtm => {
            let manager = wl_state
                .hyprland_manager
                .as_ref()
                .ok_or_else(|| "hyprland manager missing".to_string())?;

            for output in wl_state.outputs.values() {
                let matrix = if output.is_excluded(&filter.excluded_ids) {
                    color::identity_matrix()
                } else {
                    color::ctm_matrix(filter.temperature_k, filter.gamma_pct, filter.identity)
                };

                hyprland_ctm::apply_matrix(manager, &output.wl_output, matrix);
            }

            hyprland_ctm::commit(manager);
            Ok(())
        }
        BackendKind::WlrGamma => {
            let output_ids: Vec<u32> = wl_state.outputs.keys().copied().collect();
            let mut errors = Vec::new();

            for global_id in output_ids {
                let Some(output) = wl_state.outputs.get(&global_id) else {
                    continue;
                };
                let Some(control) = wl_state.wlr_controls.get(&global_id) else {
                    continue;
                };

                if control.failed {
                    continue;
                }
                if control.gamma_size.is_none() {
                    continue;
                }

                let multipliers = if output.is_excluded(&filter.excluded_ids) {
                    [1.0, 1.0, 1.0]
                } else {
                    color::channel_multipliers(
                        filter.temperature_k,
                        filter.gamma_pct,
                        filter.identity,
                    )
                };

                if let Err(err) = wlr_gamma::apply_control(control, multipliers) {
                    let output_id = output.primary_id();
                    errors.push(format!("{output_id}: {err}"));
                }
            }

            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors.join("; "))
            }
        }
    }
}

struct RequestResult {
    response: String,
    should_stop: bool,
    needs_apply: bool,
}

fn handle_request(
    line: &str,
    filter: &mut FilterState,
    wl_state: &mut WaylandState,
    backend: BackendKind,
) -> RequestResult {
    let parsed = match ControlRequest::from_wire(line) {
        Ok(req) => req,
        Err(err) => {
            return RequestResult {
                response: format!("error: {err}"),
                should_stop: false,
                needs_apply: false,
            };
        }
    };

    match parsed {
        ControlRequest::SetTemperature(change) => {
            match resolve_temperature(filter.temperature_k, change) {
                Ok(new_temp) => {
                    filter.temperature_k = new_temp;
                    filter.identity = false;
                    RequestResult {
                        response: "ok".to_string(),
                        should_stop: false,
                        needs_apply: true,
                    }
                }
                Err(err) => RequestResult {
                    response: format!("error: {err}"),
                    should_stop: false,
                    needs_apply: false,
                },
            }
        }
        ControlRequest::SetGamma(change) => match resolve_gamma(filter.gamma_pct, change) {
            Ok(new_gamma) => {
                filter.gamma_pct = new_gamma;
                RequestResult {
                    response: "ok".to_string(),
                    should_stop: false,
                    needs_apply: true,
                }
            }
            Err(err) => RequestResult {
                response: format!("error: {err}"),
                should_stop: false,
                needs_apply: false,
            },
        },
        ControlRequest::SetIdentity(value) => {
            match value {
                IdentityValue::True => filter.identity = true,
                IdentityValue::False => filter.identity = false,
                IdentityValue::Toggle => filter.identity = !filter.identity,
            }
            RequestResult {
                response: "ok".to_string(),
                should_stop: false,
                needs_apply: true,
            }
        }
        ControlRequest::Get(field) => {
            let response = match field {
                GetField::Temperature => format!("ok temperature={}", filter.temperature_k),
                GetField::Gamma => format!("ok gamma={}", format_float(filter.gamma_pct)),
                GetField::Identity => format!("ok identity={}", filter.identity),
                GetField::Backend => format!("ok backend={backend}"),
                GetField::State => format!(
                    "ok state=temperature:{} gamma:{} identity:{} backend:{} excludes:{}",
                    filter.temperature_k,
                    format_float(filter.gamma_pct),
                    filter.identity,
                    backend,
                    join_csv(filter.excluded_ids.iter().cloned().collect()),
                ),
            };
            RequestResult {
                response,
                should_stop: false,
                needs_apply: false,
            }
        }
        ControlRequest::Reset => {
            filter.reset_to_defaults();
            RequestResult {
                response: "ok".to_string(),
                should_stop: false,
                needs_apply: true,
            }
        }
        ControlRequest::OutputsList => {
            let mut ids: Vec<String> = wl_state
                .outputs
                .values()
                .map(|output| {
                    let id = output.primary_id();
                    if output.is_excluded(&filter.excluded_ids) {
                        format!("{id}*")
                    } else {
                        id
                    }
                })
                .collect();
            ids.sort();
            RequestResult {
                response: format!("ok outputs={}", join_csv(ids)),
                should_stop: false,
                needs_apply: false,
            }
        }
        ControlRequest::ExcludeAdd(id) => {
            filter.excluded_ids.insert(id);
            RequestResult {
                response: "ok".to_string(),
                should_stop: false,
                needs_apply: true,
            }
        }
        ControlRequest::ExcludeRemove(id) => {
            filter.excluded_ids.remove(&id);
            RequestResult {
                response: "ok".to_string(),
                should_stop: false,
                needs_apply: true,
            }
        }
        ControlRequest::ExcludeList => {
            let values: Vec<String> = filter.excluded_ids.iter().cloned().collect();
            RequestResult {
                response: format!("ok excludes={}", join_csv(values)),
                should_stop: false,
                needs_apply: false,
            }
        }
        ControlRequest::Stop => RequestResult {
            response: "ok".to_string(),
            should_stop: true,
            needs_apply: false,
        },
    }
}

fn resolve_temperature(current: u32, change: TemperatureChange) -> Result<u32, String> {
    let candidate = match change {
        TemperatureChange::Absolute(value) => value,
        TemperatureChange::Relative(delta) => current as i64 + delta,
    };

    if !(1000..=20000).contains(&candidate) {
        return Err("temperature must be in range 1000..=20000".to_string());
    }

    Ok(candidate as u32)
}

fn resolve_gamma(current: f64, change: GammaChange) -> Result<f64, String> {
    let candidate = match change {
        GammaChange::Absolute(value) => value,
        GammaChange::Relative(delta) => current + delta,
    };

    if !(0.0..=200.0).contains(&candidate) {
        return Err("gamma must be in range 0..=200".to_string());
    }

    Ok(candidate)
}

fn validate_temperature(value: u32) -> Result<(), String> {
    if !(1000..=20000).contains(&value) {
        return Err("temperature must be in range 1000..=20000".to_string());
    }
    Ok(())
}

fn validate_gamma(value: f64) -> Result<(), String> {
    if !(0.0..=200.0).contains(&value) {
        return Err("gamma must be in range 0..=200".to_string());
    }
    Ok(())
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

fn join_csv(values: Vec<String>) -> String {
    if values.is_empty() {
        "-".to_string()
    } else {
        values.join(",")
    }
}

fn vlog(verbose: bool, message: &str) {
    if verbose {
        eprintln!("[nighterrors] {message}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temperature_bounds() {
        assert!(resolve_temperature(6000, TemperatureChange::Absolute(999)).is_err());
        assert!(resolve_temperature(6000, TemperatureChange::Absolute(20001)).is_err());
        assert_eq!(
            resolve_temperature(6000, TemperatureChange::Relative(-500)).unwrap(),
            5500
        );
    }

    #[test]
    fn gamma_bounds() {
        assert!(resolve_gamma(100.0, GammaChange::Absolute(-1.0)).is_err());
        assert!(resolve_gamma(100.0, GammaChange::Absolute(201.0)).is_err());
        assert_eq!(
            resolve_gamma(100.0, GammaChange::Relative(5.5)).unwrap(),
            105.5
        );
    }

    #[test]
    fn float_formatting_trims_trailing_zeros() {
        assert_eq!(format_float(100.0), "100");
        assert_eq!(format_float(95.5), "95.5");
    }

    #[test]
    fn set_request_requires_apply() {
        let mut filter = FilterState {
            temperature_k: 6000,
            gamma_pct: 100.0,
            identity: false,
            excluded_ids: BTreeSet::new(),
        };
        let mut wl_state = WaylandState::new(false);

        let result = handle_request(
            "set temperature 5500",
            &mut filter,
            &mut wl_state,
            BackendKind::WlrGamma,
        );

        assert_eq!(result.response, "ok");
        assert!(result.needs_apply);
        assert!(!result.should_stop);
        assert_eq!(filter.temperature_k, 5500);
    }

    #[test]
    fn get_request_does_not_require_apply() {
        let mut filter = FilterState {
            temperature_k: 6000,
            gamma_pct: 100.0,
            identity: false,
            excluded_ids: BTreeSet::new(),
        };
        let mut wl_state = WaylandState::new(false);

        let result = handle_request(
            "get temperature",
            &mut filter,
            &mut wl_state,
            BackendKind::HyprlandCtm,
        );

        assert_eq!(result.response, "ok temperature=6000");
        assert!(!result.needs_apply);
        assert!(!result.should_stop);
    }
}
