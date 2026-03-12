use crate::CONSOLE;

#[macro_export]
macro_rules! console_print {
    ($($arg:tt)*) => ($crate::render::print::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! console_println {
    () => ($crate::console_print!("\n"));
    ($($arg:tt)*) => ($crate::console_print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;

    x86_64::instructions::interrupts::without_interrupts(|| {
        CONSOLE.get().unwrap().lock().write_fmt(args).unwrap();
    })
}
