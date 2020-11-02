mod app;
mod bindings;
mod buttons;
mod playlists;

fn main() {
    bindings::init();

    yew::start_app::<app::App>();
}
