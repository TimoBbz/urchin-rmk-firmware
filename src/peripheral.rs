#![no_main]
#![no_std]

use defmt::unwrap;
use embassy_nrf::{
    gpio::{Level, Output, OutputDrive},
    peripherals, spim,
};
use rmk::{
    channel::{ControllerSub, CONTROLLER_CHANNEL},
    controller::Controller,
    event::ControllerEvent,
};
use sharp_memory_display::*;

use rmk::macros::rmk_peripheral;

mod no_pin;

use crate::no_pin::NoPin;

struct ScreenController {
    sub: ControllerSub,
}

impl Controller for ScreenController {
    type Event = ControllerEvent;

    async fn process_event(&mut self, _event: Self::Event) {}

    async fn next_message(&mut self) -> Self::Event {
        self.sub.next_message_pure().await
    }
}

const DISP: NoPin = NoPin;

#[rmk_peripheral(id = 0)]
mod keyboard_peripheral {
    #[controller(event)]
    fn screen_controller() -> ScreenController {
        bind_interrupts!(struct Irqs {
            SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
        });
        let mut config = spim::Config::default();
        config.mode = MODE;
        let spi = spim::Spim::new_txonly(p.SPI3, Irqs, p.P0_20, p.P0_17, config);
        let cs = Output::new(p.P0_06, Level::High, OutputDrive::Standard);
        let mut display = MemoryDisplay::new(spi, cs, DISP);
        display.clear();

        ScreenController {
            sub: unwrap!(CONTROLLER_CHANNEL.subscriber()),
        }
    }
}
