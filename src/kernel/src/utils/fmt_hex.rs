use core::fmt;

pub fn fmt_hex<T>(value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result
where
    T: fmt::LowerHex,
{
    write!(f, "{:#x}", value)
}
