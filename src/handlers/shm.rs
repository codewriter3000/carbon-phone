use smithay::wayland::shm::{ShmHandler, ShmState, ShmPoolUserData};
use wayland_server::{
    protocol::{wl_shm::WlShm, wl_shm_pool::WlShmPool},
    Dispatch, DisplayHandle, Client,
};

use crate::state::State;

impl ShmHandler for State {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
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


