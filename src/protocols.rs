pub mod hyprland_ctm {
    use wayland_client;
    use wayland_client::protocol::*;

    pub mod __interfaces {
        use wayland_client::backend as wayland_backend;
        use wayland_client::protocol::__interfaces::*;
        wayland_scanner::generate_interfaces!("./protocols/hyprland-ctm-control-v1.xml");
    }
    use self::__interfaces::*;

    wayland_scanner::generate_client_code!("./protocols/hyprland-ctm-control-v1.xml");
}

pub mod wlr_gamma {
    use wayland_client;
    use wayland_client::protocol::*;

    pub mod __interfaces {
        use wayland_client::backend as wayland_backend;
        use wayland_client::protocol::__interfaces::*;
        wayland_scanner::generate_interfaces!("./protocols/wlr-gamma-control-unstable-v1.xml");
    }
    use self::__interfaces::*;

    wayland_scanner::generate_client_code!("./protocols/wlr-gamma-control-unstable-v1.xml");
}
