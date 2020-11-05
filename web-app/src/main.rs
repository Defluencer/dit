mod app;
mod bindings;
mod live_stream_manager;
mod live_stream_player;
mod playlists;

fn main() {
    yew::start_app::<app::App>();
}
