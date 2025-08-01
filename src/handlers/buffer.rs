use smithay::wayland::buffer::BufferHandler;
use crate::state::State;

impl BufferHandler for State {
    fn buffer_destroyed(&mut self, _buffer: &wayland_server::protocol::wl_buffer::WlBuffer) {
        // Optional
    }
}

