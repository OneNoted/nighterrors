use std::ffi::CString;
use std::io;
use std::os::fd::BorrowedFd;
use std::os::fd::RawFd;

use wayland_client::protocol::wl_output;
use wayland_client::{Dispatch, QueueHandle};

use crate::color;
use crate::protocols::wlr_gamma::zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1;
use crate::protocols::wlr_gamma::zwlr_gamma_control_v1::ZwlrGammaControlV1;

#[derive(Debug, Clone)]
pub struct WlrControlState {
    pub control: ZwlrGammaControlV1,
    pub gamma_size: Option<u32>,
    pub failed: bool,
}

impl WlrControlState {
    pub fn new(control: ZwlrGammaControlV1) -> Self {
        Self {
            control,
            gamma_size: None,
            failed: false,
        }
    }
}

pub fn create_control<D>(
    manager: &ZwlrGammaControlManagerV1,
    output: &wl_output::WlOutput,
    qh: &QueueHandle<D>,
    output_global_id: u32,
) -> WlrControlState
where
    D: Dispatch<ZwlrGammaControlV1, u32> + 'static,
{
    let control = manager.get_gamma_control(output, qh, output_global_id);
    WlrControlState::new(control)
}

pub fn destroy_control(control: &WlrControlState) {
    control.control.destroy();
}

pub fn apply_control(control: &WlrControlState, multipliers: [f64; 3]) -> Result<(), String> {
    if control.failed {
        return Err("wlr gamma control object is marked as failed".to_string());
    }

    let gamma_size = control
        .gamma_size
        .ok_or_else(|| "gamma size has not been announced yet".to_string())?;

    if gamma_size < 2 {
        return Err(format!(
            "invalid gamma ramp size {gamma_size}, expected at least 2"
        ));
    }

    let table = color::build_gamma_lut(gamma_size as usize, multipliers);
    let fd = memfd_create("nighterrors-gamma")?;

    let apply_result = (|| {
        write_all_fd(fd, &table)?;

        let seek_result = unsafe { libc::lseek(fd, 0, libc::SEEK_SET) };
        if seek_result < 0 {
            return Err(io::Error::last_os_error().to_string());
        }

        let borrowed = unsafe { BorrowedFd::borrow_raw(fd) };
        control.control.set_gamma(borrowed);

        Ok(())
    })();

    let _ = unsafe { libc::close(fd) };
    apply_result
}

fn memfd_create(name: &str) -> Result<RawFd, String> {
    let cname = CString::new(name).map_err(|e| e.to_string())?;
    let fd = unsafe { libc::memfd_create(cname.as_ptr(), libc::MFD_CLOEXEC) };
    if fd < 0 {
        return Err(io::Error::last_os_error().to_string());
    }
    Ok(fd)
}

fn write_all_fd(fd: RawFd, mut data: &[u8]) -> Result<(), String> {
    while !data.is_empty() {
        let written = unsafe { libc::write(fd, data.as_ptr().cast(), data.len()) };
        if written < 0 {
            return Err(io::Error::last_os_error().to_string());
        }

        let written = written as usize;
        if written == 0 {
            return Err("short write while writing gamma table".to_string());
        }

        data = &data[written..];
    }

    Ok(())
}
