use std::convert::TryFrom;

use web_sys::{Storage, Window};

use yew::services::ConsoleService;

use cid::Cid;

const VIDEO_LIST_CID_LOCAL_KEY: &str = "video_list_cid";

pub fn get_local_storage(window: &Window) -> Option<Storage> {
    #[cfg(debug_assertions)]
    ConsoleService::info("Get Local Storage");

    match window.local_storage() {
        Ok(option) => option,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            None
        }
    }
}

pub fn get_local_list(storage: Option<&Storage>) -> Option<Cid> {
    let storage = match storage {
        Some(st) => st,
        None => return None,
    };

    let cid = match storage.get_item(VIDEO_LIST_CID_LOCAL_KEY) {
        Ok(option) => option,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return None;
        }
    };

    let cid = cid?;

    let cid = Cid::try_from(cid).expect("Invalid Cid");

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!(
        "Storage Get => {} \n {}",
        VIDEO_LIST_CID_LOCAL_KEY,
        &cid.to_string()
    ));

    Some(cid)
}

pub fn set_local_list(cid: &Cid, storage: Option<&Storage>) {
    let storage = match storage {
        Some(st) => st,
        None => return,
    };

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!(
        "Storage Set => {} \n {}",
        VIDEO_LIST_CID_LOCAL_KEY,
        &cid.to_string()
    ));

    if let Err(e) = storage.set_item(VIDEO_LIST_CID_LOCAL_KEY, &cid.to_string()) {
        ConsoleService::error(&format!("{:?}", e));
    }
}
