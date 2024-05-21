use alloc::format;
use alloc::string::String;
use core::{cmp};

pub fn human_bytes(num: impl Into<f64>) -> String {
    let num = num.into();
    let negative = if num.is_sign_positive() { "" } else { "-" };
    let num = libm::fabs(num);
    
    const UNITS: [&str; 9] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB"];
    if num < 1_f64 {
        return format!("{}{} {}", negative, num, "B");
    }
    let delimiter = 1024_f64;
    let exponent = cmp::min(libm::floor(libm::log(num) / libm::log(delimiter)) as i32, (UNITS.len() - 1) as i32);
    let pretty_bytes = format!("{:.2}", num / libm::pow(delimiter, exponent as f64)).parse::<f64>().unwrap() * 1_f64;
    let unit = UNITS[exponent as usize];
    format!("{}{} {}", negative, pretty_bytes, unit)
}