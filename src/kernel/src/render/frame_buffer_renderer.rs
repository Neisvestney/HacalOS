use bootloader_structs::GopInfo;

struct FrameBufferRenderer {
    pub frame_buffer: *mut u8,
    pub frame_buffer_size: usize,
    pub horizontal_resolution: usize,
    pub vertical_resolution: usize,
}

impl FrameBufferRenderer {
    pub fn new(gop_info: GopInfo) -> Self {
        FrameBufferRenderer {
            frame_buffer: gop_info.frame_buffer,
            frame_buffer_size: gop_info.frame_buffer_size,
            horizontal_resolution: gop_info.horizontal_resolution,
            vertical_resolution: gop_info.vertical_resolution,
        }
    }
}