use crate::render::color::Color;
use crate::{CONSOLE, println, serial_println};
use core::fmt::Write;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

static LOGGER: SimpleLogger = SimpleLogger;

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            x86_64::instructions::interrupts::without_interrupts(|| {
                let mut console = CONSOLE.get().unwrap().lock(); // TODO Rewrite with ascii control characters
                console.set_foreground_color(Color::GRAY);
                console.write_char('[');
                console.set_foreground_color(get_level_color(record.level()));
                console.write_fmt(format_args!("{}", record.level()));
                console.set_foreground_color(Color::GRAY);
                console.write_string("] ");
                console.set_foreground_color(Color::WHITE);
                console.write_fmt(format_args!("{}\n", record.args()));
                serial_println!("[{}] {}", record.level(), record.args());
            });
            // println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init_logger() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}

fn get_level_color(level: Level) -> Color {
    match level {
        Level::Error => Color::ERROR,
        Level::Warn => Color::YELLOW,
        Level::Info => Color::LIGHT_GRAY,
        Level::Debug => Color::GRAY,
        Level::Trace => Color::GRAY,
    }
}
