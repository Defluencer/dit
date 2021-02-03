use std::convert::TryFrom;

use web_sys::{Storage, Window};

use yew::services::ConsoleService;

use cid::Cid;

// TODO turn this into a struct?

pub fn get_local_storage(window: &Window) -> Option<Storage> {
    #[cfg(debug_assertions)]
    ConsoleService::info("Get Local Storage");

    match window.local_storage() {
        Ok(option) => option,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            None
        }
    }
}

pub fn get_local_list(ipns_hash: &str, storage: Option<&Storage>) -> Option<Cid> {
    let storage = storage?;

    let cid = match storage.get_item(ipns_hash) {
        Ok(option) => option,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return None;
        }
    };

    let cid = cid?;

    let cid = Cid::try_from(cid).expect("Invalid Cid");

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!(
        "Storage Get => {} \n {}",
        ipns_hash,
        &cid.to_string()
    ));

    Some(cid)
}

pub fn set_local_list(ipns_hash: &str, cid: &Cid, storage: Option<&Storage>) {
    let storage = match storage {
        Some(st) => st,
        None => return,
    };

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!(
        "Storage Set => {} \n {}",
        ipns_hash,
        &cid.to_string()
    ));

    if let Err(e) = storage.set_item(ipns_hash, &cid.to_string()) {
        ConsoleService::error(&format!("{:#?}", e));
    }
}
