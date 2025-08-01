use smithay::{
    wayland::{
        compositor::{CompositorHandler, CompositorState, CompositorClientState},
    },
    reexports::{
        wayland_server::{Client, backend::ClientData, protocol::wl_surface::WlSurface},
    },
};
use crate::state::{State, ClientState};
use wayland_server::backend::{ClientId, DisconnectReason};

impl CompositorHandler for State {
    fn commit(&mut self, _surface: &WlSurface) {}

    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(
        &self,
        client: &'a Client,
    ) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().expect("ClientState missing for client").compositor_state
    }
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}
