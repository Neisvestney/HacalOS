#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Ord, PartialOrd)]
pub struct Color(u32);

impl Color {
    pub const fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Color((red as u32) << 16 | (green as u32) << 8 | blue as u32)
    }
    
    pub const fn as_u32(&self) -> u32 {
        self.0
    }
}