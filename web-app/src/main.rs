#![recursion_limit = "1024"]

mod agents;
mod app;
mod bindings;
mod components;
mod pages;
mod playlists;

fn main() {
    yew::start_app::<app::App>();
}
