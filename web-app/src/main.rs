#![recursion_limit = "1024"]

mod app;
mod components;
mod pages;
mod utils;

use std::rc::Rc;

use crate::app::Props;
use crate::utils::{IpfsService, LocalStorage, Web3Service};

const ENS_NAME: &str = "sionois";

fn main() {
    let web3 = Web3Service::new();
    let storage = LocalStorage::new();
    let ipfs = IpfsService::new(&storage);
    let ens_name = Rc::from(ENS_NAME);

    let props = Props {
        web3,
        ipfs,
        storage,
        ens_name,
    };

    yew::start_app_with_props::<app::App>(props);
}
