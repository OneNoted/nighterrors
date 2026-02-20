fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=protocols/hyprland-ctm-control-v1.xml");
    println!("cargo:rerun-if-changed=protocols/wlr-gamma-control-unstable-v1.xml");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "linux" {
        panic!("nighterrors currently supports Linux only");
    }
}
