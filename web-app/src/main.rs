mod app;
mod bindings;
mod live_stream;
mod playlists;
mod video;

fn main() {
    yew::start_app::<app::App>();
}
