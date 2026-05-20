# What is LeafShell
LeafShell is the Hyprland shell that will be used in ArcOS
the Arc Desktop Environment will look as follows:
```
      ArcOS Compoenents -> Applications like Settings, PDF viewer, phone sync apps, etc..
---------------------------
         LeafShell -> The core shell: Notification manager, Topbar, Tophub, (Maybe even Dock)
---------------------------
         Hyprland -> The window manager, it will be controlled via internal .lua scripts with the new config
                        system to fully integrate it like if we made our own compositor
```

# Why Rust/Iced
I looked into other languages but found a few uninteresting to me:
C: too raw, can't get stuff done quickly enough
C++: RAII is a timed neuclear bomb
Zig: Still too new, I did try to contribute to `dvui` and talk to david (the creator)
     But never actually agreed because this is too much for `dvui` at this early stage
Others: I'm lazy tbh I already new rust and liked iced so had to simplify things and decrease the friction

Why iced?
Simply because all other GUI libraries in the rust ecosystem are trying to be a shitier react

# Developers
- Me (KernelState)

I'd like to see designers and developers alike giving me suggestions or even contributing code 

# Code Rules
These are rules for contriuting to this project and the entire ArcOS ecosystem in general
All User-Facing GUI desktop applications DO NOT maintain state nor Sync it with other state machines.
State and data are fetched from state machines or services that is used between each update, GUI Applications
should use a tick system to fetch the information to avoid deadlocks presented by polling.
Heavy work should be offloaded from the update function to other tokio workers that can later pipe into other
Messages into the update function to help with performance
