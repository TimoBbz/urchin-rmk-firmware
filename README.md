# Urchin RMK Ergo-L

A firmware for the [Urchin Keyboard](https://github.com/duckyb/urchin), made with [RMK](https://rmk.rs/) to use with the layout [Ergo-L](https://ergol.org/).

## Install instructions

- Install the layout Ergo-L on your machine
- Build and flash the firmware to the keyboard
   1. `cargo make uf2 --release`
   2. Flash each uf2 file to its keyboard part (central is left), with drag-&-drop.

## Debugging

To understand how the controller event works without a debug probe, the log_controller module can be used.

1. Add heapless to the project: `cargo add heapless`
2. Replace the ScreenController with the LogController:
```rs
mod log_controller;
use log_controller::LogController;

#[rmk_central]
mod keyboard_central {
    #[controller(event)]
    fn screen_controller() -> LogController {
        /*
        ...
        */

        LogController {
            sub: unwrap!(CONTROLLER_CHANNEL.subscriber()),
            display,
            log_history: [None; LOG_LINES],
        }
    }
}
```