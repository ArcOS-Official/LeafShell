mod audio;
mod hyprland;
mod media;
mod network;
mod topbar;

fn main() {
    topbar::main().expect("Topbar failed to start");
}
