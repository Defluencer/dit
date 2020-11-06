mod app;
mod bindings;
mod components;
mod live_stream_manager;
mod playlists;
mod routing;
mod vod_manager;

fn main() {
    yew::start_app::<app::App>();
}
