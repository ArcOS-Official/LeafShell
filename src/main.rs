mod audio;
mod hyprland;
mod network;
mod topbar;
mod media;

fn main() {
    topbar::main().expect("Topbar failed to start");
}
