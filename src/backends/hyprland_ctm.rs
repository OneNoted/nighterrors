use wayland_client::protocol::wl_output;

use crate::protocols::hyprland_ctm::hyprland_ctm_control_manager_v1::HyprlandCtmControlManagerV1;

pub fn apply_matrix(
    manager: &HyprlandCtmControlManagerV1,
    output: &wl_output::WlOutput,
    matrix: [f64; 9],
) {
    manager.set_ctm_for_output(
        output, matrix[0], matrix[1], matrix[2], matrix[3], matrix[4], matrix[5], matrix[6],
        matrix[7], matrix[8],
    );
}

pub fn commit(manager: &HyprlandCtmControlManagerV1) {
    manager.commit();
}
