use std::sync::Arc;
use smithay::{
    delegate_compositor, delegate_shm,
    wayland::{
        compositor::{CompositorClientState, CompositorHandler, CompositorState},
        shm::{ShmHandler, ShmState, ShmPoolUserData},
        buffer::BufferHandler,
        socket::ListeningSocketSource,
    },
    backend::input::InputEvent,
    reexports::{
        calloop::EventLoop,
        wayland_server::{
            backend::{ClientData, ClientId},
            protocol::{wl_compositor::WlCompositor, wl_shm::WlShm, wl_shm_pool::WlShmPool, wl_subcompositor::WlSubcompositor, wl_surface::WlSurface},
            Client, Display, DisplayHandle, GlobalDispatch, Dispatch,
        },
    },
};
use slog::{Drain, Logger};
use wayland_server::backend::DisconnectReason;
use wayland_server::protocol::wl_buffer::WlBuffer;

struct State {
    compositor_state: CompositorState,
    shm_state: ShmState,
}

struct ClientState {
    compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

impl GlobalDispatch<WlCompositor, ()> for State {
    fn bind(
        _state: &mut State,
        _handle: &DisplayHandle,
        _client: &Client,
        _resource: wayland_server::New<WlCompositor>,
        _global_data: &(),
        _data_init: &mut wayland_server::DataInit<'_, State>,
    ) {
        // nothing needed - compositor client state is set automatically
    }
}

impl GlobalDispatch<WlSubcompositor, ()> for State {
    fn bind(
        _state: &mut State,
        _handle: &DisplayHandle,
        _client: &Client,
        _resource: wayland_server::New<WlSubcompositor>,
        _global_data: &(),
        _data_init: &mut wayland_server::DataInit<'_, State>,
    ) {}
}

impl GlobalDispatch<WlShm, ()> for State {
    fn bind(
        _state: &mut State,
        _handle: &DisplayHandle,
        _client: &Client,
        _resource: wayland_server::New<WlShm>,
        _global_data: &(),
        _data_init: &mut wayland_server::DataInit<'_, State>,
    ) {}
}

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


impl ShmHandler for State {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl BufferHandler for State {
    fn buffer_destroyed(&mut self, _buffer: &wayland_server::protocol::wl_buffer::WlBuffer) {
        // Optional
    }
}

impl Dispatch<WlShm, ()> for State {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &WlShm,
        _request: <WlShm as wayland_server::Resource>::Request,
        _data: &(),
        _dh: &DisplayHandle,
        _data_init: &mut wayland_server::DataInit<'_, State>,
    ) {}
}

impl Dispatch<WlShmPool, ShmPoolUserData> for State {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &WlShmPool,
        _request: <WlShmPool as wayland_server::Resource>::Request,
        _data: &ShmPoolUserData,
        _dh: &DisplayHandle,
        _data_init: &mut wayland_server::DataInit<'_, State>,
    ) {}
}


//delegate_compositor!(State);
//delegate_shm!(State);

impl ClientData for State {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

fn main() {
    // Logging
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = Logger::root(drain, slog::o!());

    // Wayland display
    let mut display = Display::<State>::new().unwrap();
    let mut dh: DisplayHandle = display.handle();

    // Register compositor and shm globals
    let comp = CompositorState::new::<State>(&dh);
    let shm = ShmState::new::<State>(&dh, vec![]);

    let mut state = State {
        compositor_state: comp,
        shm_state: shm,
    };

    // Listening socket
    let socket = ListeningSocketSource::new_auto().unwrap();
    println!("Listening on socket: {}", socket.socket_name().to_string_lossy());

    // Event loop
    let mut event_loop: EventLoop<'static, State> = EventLoop::try_new().unwrap();
    event_loop
        .handle()
        .insert_source(socket, move |stream, _, state| {
            dh.insert_client(stream, Arc::new(()))
                .expect("Failed to insert client");
        })
        .expect("Failed to register socket");

    // Main loop
    loop {
        event_loop.dispatch(None, &mut state).unwrap();
        display.dispatch_clients(&mut state).unwrap();
        display.flush_clients().unwrap();
    }
}
