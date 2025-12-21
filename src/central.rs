#![no_main]
#![no_std]
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
    prelude::{Dimensions, DrawTarget, Point, Size},
    primitives::Rectangle,
    text::Text,
    Drawable, Pixel,
};
use rmk::{
    ble::BleState,
    channel::{ControllerSub, CONTROLLER_CHANNEL},
    controller::Controller,
    event::ControllerEvent,
    macros::rmk_central,
};
use sharp_memory_display::*;

use crate::no_pin::NoPin;

mod no_pin;

#[derive(Default, PartialEq)]
enum MyBleState {
    Advertising,
    Connected,
    #[default]
    None,
}

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

pub struct RotatedDrawTarget<T>
where
    T: DrawTarget,
{
    parent: T,
}

impl<T> RotatedDrawTarget<T>
where
    T: DrawTarget,
{
    pub fn new(parent: T) -> Self {
        Self { parent }
    }
}

impl<T> DrawTarget for RotatedDrawTarget<T>
where
    T: DrawTarget,
{
    type Color = T::Color;
    type Error = T::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let parent_width = self.parent.bounding_box().size.width as i32;

        self.parent.draw_iter(
            pixels
                .into_iter()
                .map(|Pixel(p, c)| Pixel(Point::new(parent_width - p.y, p.x), c)),
        )
    }
}

impl<T> Dimensions for RotatedDrawTarget<T>
where
    T: DrawTarget,
{
    fn bounding_box(&self) -> Rectangle {
        let parent_bb = self.parent.bounding_box();
        Rectangle::new(
            parent_bb.top_left,
            Size::new(parent_bb.size.height, parent_bb.size.width),
        )
    }
}

#[derive(Default)]
struct ScreenState {
    layer: u8,
    ble_profile: u8,
    ble_state: MyBleState,
    connection_type: u8,
    battery_percent: u8,
    charging_state: bool,
}

struct ScreenController<'a> {
    sub: ControllerSub,
    display: RotatedDrawTarget<MemoryDisplay<spim::Spim<'a>, Output<'a>, NoPin>>,
    current_state: ScreenState,
}

impl ScreenController<'_> {
    fn flush_state_to_the_display(&mut self) {
        self.display.parent.clear_buffer();

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
                    MyBleState::Advertising => BLUETOOTH_ADVERTISING_DATA,
                    MyBleState::Connected => BLUETOOTH_CONNECTED_DATA,
                    MyBleState::None => BLUETOOTH_NONE_DATA,
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

        self.display.parent.flush_buffer();
    }
}

impl Controller for ScreenController<'_> {
    type Event = ControllerEvent;

    async fn process_event(&mut self, event: Self::Event) {
        match event {
            ControllerEvent::Layer(layer) => {
                if layer == self.current_state.layer {
                    return;
                }
                self.current_state.layer = layer;
            }
            ControllerEvent::Battery(battery_percent) => {
                if battery_percent == self.current_state.battery_percent {
                    return;
                }
                self.current_state.battery_percent = battery_percent;
            }
            ControllerEvent::BleState(profile, ble_state) => {
                let my_ble_state = match ble_state {
                    BleState::Advertising => MyBleState::Advertising,
                    BleState::Connected => MyBleState::Connected,
                    BleState::None => MyBleState::None,
                };
                if my_ble_state == self.current_state.ble_state
                    && profile == self.current_state.ble_profile
                {
                    return;
                }
                self.current_state.ble_profile = profile;
                self.current_state.ble_state = my_ble_state;
            }
            ControllerEvent::BleProfile(profile) => {
                if profile == self.current_state.ble_profile {
                    return;
                }
                self.current_state.ble_profile = profile;
            }
            ControllerEvent::ChargingState(state) => {
                if state == self.current_state.charging_state {
                    return;
                }
                self.current_state.charging_state = state;
            }
            ControllerEvent::ConnectionType(connection_type) => {
                if self.current_state.connection_type == connection_type {
                    return;
                }
                self.current_state.connection_type = connection_type;
            }
            _ => (),
        }
        self.flush_state_to_the_display();
    }

    async fn next_message(&mut self) -> Self::Event {
        self.sub.next_message_pure().await
    }
}

const DISP: NoPin = NoPin;

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
        let mut display = MemoryDisplay::new(spi, cs, DISP);
        display.clear();

        ScreenController {
            sub: unwrap!(CONTROLLER_CHANNEL.subscriber()),
            display: RotatedDrawTarget::new(display),
            current_state: ScreenState::default(),
        }
    }
}
