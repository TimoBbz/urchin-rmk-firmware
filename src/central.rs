#![no_main]
#![no_std]
use core::mem::{discriminant, replace};

use defmt::unwrap;
use embassy_nrf::{
    gpio::{Level, Output, OutputDrive},
    peripherals, spim,
};
use embedded_graphics::{
    image::{Image, ImageRaw},
    mono_font::{
        ascii::{FONT_10X20, FONT_7X13_BOLD, FONT_9X15},
        MonoTextStyle,
    },
    pixelcolor::BinaryColor,
    prelude::Point,
    text::Text,
    Drawable,
};
use rmk::{
    ble::BleState,
    channel::{ControllerSub, CONTROLLER_CHANNEL},
    controller::Controller,
    event::ControllerEvent,
    macros::rmk_central,
};
use sharp_memory_display::*;

use crate::nice_view::NiceView;

mod nice_view;

#[rustfmt::skip]
const BLUETOOTH_NONE_DATA: &[u8] = &[
    0b11111111, 0b01111111, 0b11_000000,
    0b11111111, 0b00111111, 0b11_000000,
    0b11111111, 0b00011111, 0b11_000000,
    0b11111111, 0b00001111, 0b11_000000,
    0b11111111, 0b00000111, 0b11_000000,
    0b11111111, 0b00100011, 0b11_000000,
    0b00111111, 0b00110001, 0b11_000000,
    0b00011111, 0b00111000, 0b11_000000,
    0b10001111, 0b00111100, 0b01_000000,
    0b11000111, 0b00111000, 0b11_000000,
    0b11100011, 0b10110001, 0b11_000000,
    0b11110001, 0b11100011, 0b11_000000,
    0b11111000, 0b11100111, 0b11_000000,
    0b11111100, 0b01111111, 0b11_000000,
    0b11111110, 0b00111111, 0b11_000000,
    0b11111110, 0b00011111, 0b11_000000,
    0b11111100, 0b00001111, 0b11_000000,
    0b11111000, 0b00000111, 0b11_000000,
    0b11110001, 0b00100011, 0b11_000000,
    0b11100011, 0b00110001, 0b11_000000,
    0b11000111, 0b00111000, 0b11_000000,
    0b11001111, 0b00111100, 0b01_000000,
    0b11111111, 0b00111000, 0b00_000000,
    0b11111111, 0b00110001, 0b11_000000,
    0b11111111, 0b00100011, 0b11_000000,
    0b11111111, 0b00000111, 0b11_000000,
    0b11111111, 0b00001111, 0b11_000000,
    0b11111111, 0b00011111, 0b11_000000,
    0b11111111, 0b00111111, 0b11_000000,
    0b11111111, 0b01111111, 0b11_000000,
];

#[rustfmt::skip]
const BLUETOOTH_ADVERTISING_DATA: &[u8] = &[
    0b11111111, 0b01111111, 0b11_000000,
    0b11111111, 0b00111111, 0b11_000000,
    0b11111111, 0b00011111, 0b11_000000,
    0b11111111, 0b00001111, 0b11_000000,
    0b11111111, 0b00000111, 0b11_000000,
    0b11111111, 0b00100011, 0b11_000000,
    0b11111111, 0b00110001, 0b11_000000,
    0b11111111, 0b00111000, 0b11_000000,
    0b11001111, 0b00111100, 0b11_000000,
    0b11000111, 0b00111000, 0b11_000000,
    0b11100011, 0b00110001, 0b11_000000,
    0b11110001, 0b00100011, 0b11_000000,
    0b11111000, 0b00000111, 0b11_000000,
    0b11111100, 0b00001111, 0b11_000000,
    0b11111110, 0b00011111, 0b11_000000,
    0b11111110, 0b00011111, 0b11_000000,
    0b11111100, 0b00001111, 0b11_000000,
    0b11111000, 0b00000111, 0b11_000000,
    0b11110001, 0b00100011, 0b11_000000,
    0b11100011, 0b00110001, 0b11_000000,
    0b11000111, 0b00111000, 0b11_000000,
    0b11001111, 0b00111100, 0b11_000000,
    0b11111111, 0b00111000, 0b11_000000,
    0b11111111, 0b00110001, 0b11_000000,
    0b11111111, 0b00100011, 0b11_000000,
    0b11111111, 0b00000111, 0b11_000000,
    0b11111111, 0b00001111, 0b11_000000,
    0b11111111, 0b00011111, 0b11_000000,
    0b11111111, 0b00111111, 0b11_000000,
    0b11111111, 0b01111111, 0b11_000000,
];

#[rustfmt::skip]
const BLUETOOTH_CONNECTED_DATA: &[u8] = &[
    0b11111111, 0b01111111, 0b11_000000,
    0b11111111, 0b00111111, 0b11_000000,
    0b11111111, 0b00011111, 0b11_000000,
    0b11111111, 0b00001111, 0b11_000000,
    0b11111111, 0b00000111, 0b11_000000,
    0b11111111, 0b00100011, 0b11_000000,
    0b11111111, 0b00110001, 0b11_000000,
    0b11111111, 0b00111000, 0b11_000000,
    0b11001111, 0b00111100, 0b11_000000,
    0b11000111, 0b00111000, 0b11_000000,
    0b11100011, 0b00110001, 0b11_000000,
    0b11110001, 0b00100011, 0b11_000000,
    0b10011000, 0b00000110, 0b01_000000,
    0b00111100, 0b00001111, 0b00_000000,
    0b01100110, 0b00011001, 0b10_000000,
    0b01100110, 0b00011001, 0b10_000000,
    0b00111100, 0b00001111, 0b00_000000,
    0b10011000, 0b00000110, 0b01_000000,
    0b11110001, 0b00100011, 0b11_000000,
    0b11100011, 0b00110001, 0b11_000000,
    0b11000111, 0b00111000, 0b11_000000,
    0b11001111, 0b00111100, 0b11_000000,
    0b11111111, 0b00111000, 0b11_000000,
    0b11111111, 0b00110001, 0b11_000000,
    0b11111111, 0b00100011, 0b11_000000,
    0b11111111, 0b00000111, 0b11_000000,
    0b11111111, 0b00001111, 0b11_000000,
    0b11111111, 0b00011111, 0b11_000000,
    0b11111111, 0b00111111, 0b11_000000,
    0b11111111, 0b01111111, 0b11_000000,
];

#[rustfmt::skip]
const USB_DATA: &[u8] = &[
    0b11111111, 0b11111001, 0b11111111, 0b111_00000,
    0b11111111, 0b11000000, 0b11111111, 0b111_00000,
    0b11111111, 0b10011001, 0b11111111, 0b111_00000,
    0b10001111, 0b00111111, 0b11111111, 0b011_00000,
    0b00000110, 0b01111111, 0b11111111, 0b001_00000,
    0b00000000, 0b00000000, 0b00000000, 0b000_00000,
    0b00000111, 0b11110011, 0b11111111, 0b001_00000,
    0b10001111, 0b11111001, 0b11111111, 0b011_00000,
    0b11111111, 0b11111100, 0b11111111, 0b111_00000,
    0b11111111, 0b11111110, 0b01100011, 0b111_00000,
    0b11111111, 0b11111111, 0b00000011, 0b111_00000,
    0b11111111, 0b11111111, 0b11000011, 0b111_00000,
];

struct ScreenState {
    layer: u8,
    ble_profile: u8,
    ble_state: BleState,
    connection_type: u8,
    battery_percent: u8,
    charging_state: bool,
}

impl Default for ScreenState {
    fn default() -> Self {
        ScreenState {
            layer: 0,
            ble_profile: 0,
            ble_state: BleState::None,
            connection_type: 0,
            battery_percent: 0,
            charging_state: false,
        }
    }
}

struct ScreenController<'a> {
    sub: ControllerSub,
    display: NiceView<'a>,
    current_state: ScreenState,
}

impl ScreenController<'_> {
    fn flush_state_to_the_display(&mut self) {
        self.display.clear_buffer();

        if self.current_state.connection_type == 0 {
            let raw_image = ImageRaw::<BinaryColor>::new(USB_DATA, 27);
            unwrap!(Image::new(&raw_image, Point { x: 2, y: 10 }).draw(&mut self.display));
        } else {
            let profile = match self.current_state.ble_profile {
                0 => "1",
                1 => "2",
                2 => "3",
                _ => "?",
            };
            let profile_style = MonoTextStyle::new(&FONT_7X13_BOLD, BinaryColor::Off);
            let raw_image = ImageRaw::<BinaryColor>::new(
                match self.current_state.ble_state {
                    BleState::Advertising => BLUETOOTH_ADVERTISING_DATA,
                    BleState::Connected => BLUETOOTH_CONNECTED_DATA,
                    BleState::None => BLUETOOTH_NONE_DATA,
                },
                18,
            );
            unwrap!(Image::new(&raw_image, Point { x: 2, y: 2 }).draw(&mut self.display));
            unwrap!(
                Text::new(profile, Point { x: 22, y: 27 }, profile_style).draw(&mut self.display)
            );
        }
        let layer = match self.current_state.layer {
            0 => "TEXTE",
            1 => "NAV",
            2 => "PROG",
            3 => "PERI",
            4 => "NAV",
            _ => "",
        };
        let profile_style = MonoTextStyle::new(&FONT_10X20, BinaryColor::Off);
        unwrap!(Text::new(layer, Point { x: 6, y: 70 }, profile_style).draw(&mut self.display));

        let battery_style = MonoTextStyle::new(&FONT_9X15, BinaryColor::Off);

        let mut battery_buffer = itoa::Buffer::new();
        let battery_repr = battery_buffer.format(self.current_state.battery_percent);
        unwrap!(
            Text::new(battery_repr, Point { x: 32, y: 32 }, battery_style).draw(&mut self.display)
        );

        self.display.flush_buffer();
    }
}

impl Controller for ScreenController<'_> {
    type Event = ControllerEvent;

    async fn process_event(&mut self, event: Self::Event) {
        let state = &mut self.current_state;
        let changed = match event {
            ControllerEvent::Layer(new) => replace(&mut state.layer, new) != new,
            ControllerEvent::Battery(new) => replace(&mut state.battery_percent, new) != new,
            ControllerEvent::BleState(new_profile, new_state) => {
                let old_state = replace(&mut state.ble_state, new_state);
                let old_profile = replace(&mut state.ble_profile, new_profile);
                discriminant(&old_state) != discriminant(&new_state) || old_profile != new_profile
            }
            ControllerEvent::BleProfile(new) => replace(&mut state.ble_profile, new) != new,
            ControllerEvent::ChargingState(new) => replace(&mut state.charging_state, new) != new,
            ControllerEvent::ConnectionType(new) => replace(&mut state.connection_type, new) != new,
            _ => false,
        };

        if changed {
            self.flush_state_to_the_display();
        }
    }

    async fn next_message(&mut self) -> Self::Event {
        self.sub.next_message_pure().await
    }
}

#[rmk_central]
mod keyboard_central {
    #[controller(event)]
    fn screen_controller() -> ScreenController {
        bind_interrupts!(struct Irqs {
            SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
        });
        let mut config = spim::Config::default();
        config.mode = MODE;
        let spi = spim::Spim::new_txonly(p.SPI3, Irqs, p.P0_20, p.P0_17, config);
        let cs = Output::new(p.P0_06, Level::High, OutputDrive::Standard);
        let mut display = NiceView::new(spi, cs);
        display.clear();

        ScreenController {
            sub: unwrap!(CONTROLLER_CHANNEL.subscriber()),
            display,
            current_state: ScreenState::default(),
        }
    }
}
