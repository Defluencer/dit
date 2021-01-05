#![recursion_limit = "1024"]

mod agents;
mod app;
mod components;
mod pages;
mod utils;

fn main() {
    yew::start_app::<app::App>();
}
