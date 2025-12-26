use defmt::unwrap;
use embedded_graphics::{
    mono_font::{ascii::FONT_4X6, MonoFont, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::Point,
    text::Text,
    Drawable,
};
use heapless::{CapacityError, String};
use rmk::{ble::BleState, channel::ControllerSub, controller::Controller, event::ControllerEvent};

use crate::nice_view::NiceView;

const LOG_LINE_HEIGHT: usize = 6;
const LOG_STYLE: MonoFont<'static> = FONT_4X6;
const LOG_LINES: usize = 25; // 160 = 6 * 25 + 5
const LOG_COLUMNS: usize = 16; // 68 = 4 * 16 + 4
type LogEntry = String<LOG_COLUMNS>;
struct LogScreenController<'a> {
    sub: ControllerSub,
    display: NiceView<'a>,
    log_history: [Option<LogEntry>; LOG_LINES],
}

impl LogScreenController<'_> {
    fn log_event(&mut self, event: ControllerEvent) -> Result<(), CapacityError> {
        match event {
            ControllerEvent::Battery(l) => {
                let mut entry = LogEntry::try_from("bat")?;
                let mut buffer = itoa::Buffer::new();
                entry.push_str(buffer.format(l))?;
                self.log(entry);
            }
            ControllerEvent::ChargingState(s) => {
                self.log(LogEntry::try_from(if s {
                    "charge oui"
                } else {
                    "charge non"
                })?);
            }
            ControllerEvent::Layer(l) => {
                self.log(LogEntry::try_from(match l {
                    0 => "layer base",
                    1 => "layer nav",
                    2 => "layer prog",
                    3 => "layer peri",
                    _ => "",
                })?);
            }
            ControllerEvent::ConnectionType(t) => {
                self.log(LogEntry::try_from(if t == 0 {
                    "conn USB"
                } else {
                    "conn BLE"
                })?);
            }
            ControllerEvent::SplitPeripheral(_, c) => {
                self.log(String::try_from(if c { "peri oui" } else { "peri non" })?);
            }
            ControllerEvent::SplitCentral(c) => {
                self.log(String::try_from(if c { "peri oui" } else { "peri non" })?);
            }
            ControllerEvent::Sleep(s) => {
                self.log(String::try_from(if s { "dodo oui" } else { "dodo non" })?);
            }
            ControllerEvent::BleState(p, s) => {
                let mut entry = LogEntry::try_from(match p {
                    0 => "prof 0 ",
                    1 => "prof 1 ",
                    2 => "prof 2 ",
                    _ => "",
                })?;
                entry.push_str(match s {
                    BleState::Advertising => "advr",
                    BleState::Connected => "conn",
                    BleState::None => "none",
                })?;
                self.log(entry);
            }
            ControllerEvent::BleProfile(p) => {
                self.log(LogEntry::try_from(match p {
                    0 => "prof 0 ",
                    1 => "prof 1 ",
                    2 => "prof 2 ",
                    _ => "",
                })?);
            }
            ControllerEvent::ClearPeer => {
                self.log(LogEntry::try_from("clear peer")?);
            }
            _ => {
                return Ok(());
            }
        }
        self.flush_logs_to_display();
        Ok(())
    }

    fn log(&mut self, entry: LogEntry) {
        self.log_history.rotate_right(1);
        self.log_history[0] = Some(entry);
    }

    fn flush_logs_to_display(&mut self) {
        let log_style = MonoTextStyle::new(&LOG_STYLE, BinaryColor::Off);
        self.display.clear_buffer();
        for (index, log) in self.log_history.iter().enumerate() {
            if let Some(text) = log {
                let y = ((index + 1) * LOG_LINE_HEIGHT) as i32;
                unwrap!(Text::new(text, Point { x: 2, y }, log_style).draw(&mut self.display));
            }
        }
        self.display.flush_buffer();
    }
}

impl Controller for LogScreenController<'_> {
    type Event = ControllerEvent;

    async fn process_event(&mut self, event: Self::Event) {
        self.log_event(event).ok();
    }

    async fn next_message(&mut self) -> Self::Event {
        self.sub.next_message_pure().await
    }
}
