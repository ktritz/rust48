// Serial port — stub implementation
// No actual serial I/O in web/Tauri builds.

pub struct Serial;

impl Serial {
    pub fn new() -> Self {
        Self
    }

    pub fn init(&mut self) {}

    pub fn set_baud(&mut self, _baud: u8) {}

    pub fn transmit_char(&mut self) {}

    pub fn receive_char(&mut self) {}
}
