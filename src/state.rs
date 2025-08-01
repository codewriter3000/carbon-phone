use smithay::wayland::{compositor::{CompositorState, CompositorClientState}, shm::ShmState};

pub struct State {
    pub compositor_state: CompositorState,
    pub shm_state: ShmState,
}

pub struct ClientState {
    pub compositor_state: CompositorClientState,
}

