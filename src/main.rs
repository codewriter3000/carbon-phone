mod state;
mod handlers;

use crate::state::{State};

use std::sync::Arc;
use smithay::{
    wayland::{
        compositor::{CompositorState},
        shm::{ShmState},
        socket::ListeningSocketSource,
    },
    reexports::{
        calloop::EventLoop,
        wayland_server::{
            protocol::{wl_compositor::WlCompositor, wl_shm::WlShm, wl_subcompositor::WlSubcompositor},
            Client, Display, DisplayHandle, GlobalDispatch,
        },
    },
};
use slog::{Drain, Logger};

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

fn main() {
    // Logging
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let _log = Logger::root(drain, slog::o!());

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
        .insert_source(socket, move |stream, _, _state| {
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
