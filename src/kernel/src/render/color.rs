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

    pub const ERROR: Color = Color::from_rgb(200, 100, 100);

    pub const RED: Color = Color::from_rgb(255, 0, 0);
    pub const GREEN: Color = Color::from_rgb(0, 255, 0);
    pub const BLUE: Color = Color::from_rgb(0, 0, 255);
    pub const WHITE: Color = Color::from_rgb(255, 255, 255);
    pub const LIGHT_GRAY: Color = Color::from_rgb(200, 200, 200);
    pub const GRAY: Color = Color::from_rgb(100, 100, 100);
    pub const BLACK: Color = Color::from_rgb(0, 0, 0);
    pub const YELLOW: Color = Color::from_rgb(255, 255, 0);
    pub const CYAN: Color = Color::from_rgb(0, 255, 255);
    pub const MAGENTA: Color = Color::from_rgb(255, 0, 255);
}
