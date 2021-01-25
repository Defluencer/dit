use std::convert::TryFrom;

use web_sys::{Storage, Window};

use yew::services::ConsoleService;

use linked_data::beacon::{VideoList, VideoMetadata};

use cid::Cid;

const VIDEO_LIST_LOCAL_KEY: &str = "video_list";
const VIDEO_LIST_CID_LOCAL_KEY: &str = "video_list_cid";

pub fn get_local_storage(window: &Window) -> Option<Storage> {
    #[cfg(debug_assertions)]
    ConsoleService::info("Get Local Storage");

    match window.local_storage() {
        Ok(option) => option,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return None;
        }
    }
}

pub fn get_local_list(storage: Option<&Storage>) -> Option<(Cid, VideoList)> {
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

    let list = match storage.get_item(VIDEO_LIST_LOCAL_KEY) {
        Ok(option) => option,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return None;
        }
    };

    let cid = cid?;
    let list = list?;

    let cid = Cid::try_from(cid).expect("Invalid Cid");

    let list = match serde_json::from_str(&list) {
        Ok(list) => list,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return None;
        }
    };

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!(
        "Storage Get => {} \n {}",
        &cid.to_string(),
        &serde_json::to_string_pretty(&list).expect("Can't print")
    ));

    Some((cid, list))
}

pub fn set_local_list(cid: &Cid, list: &VideoList, storage: Option<&Storage>) {
    let storage = match storage {
        Some(st) => st,
        None => return,
    };

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!(
        "Storage Set => {} \n {}",
        &cid.to_string(),
        &serde_json::to_string_pretty(&list).expect("Can't print")
    ));

    let item = match serde_json::to_string(list) {
        Ok(s) => s,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    if let Err(e) = storage.set_item(VIDEO_LIST_CID_LOCAL_KEY, &cid.to_string()) {
        ConsoleService::error(&format!("{:?}", e));
    }

    if let Err(e) = storage.set_item(VIDEO_LIST_LOCAL_KEY, &item) {
        ConsoleService::error(&format!("{:?}", e));
    }
}

pub fn get_local_video_metadata(cid: &Cid, storage: Option<&Storage>) -> Option<VideoMetadata> {
    let storage = match storage {
        Some(st) => st,
        None => return None,
    };

    let item = match storage.get_item(&cid.to_string()) {
        Ok(option) => option,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return None;
        }
    };

    let item = item?;

    let metadata = match serde_json::from_str(&item) {
        Ok(md) => md,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return None;
        }
    };

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!(
        "Storage Get => {} \n {}",
        &cid.to_string(),
        &serde_json::to_string_pretty(&metadata).expect("Can't print")
    ));

    Some(metadata)
}

pub fn set_local_video_metadata(cid: &Cid, metadata: &VideoMetadata, storage: Option<&Storage>) {
    let storage = match storage {
        Some(st) => st,
        None => return,
    };

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!(
        "Storage Set => {} \n {}",
        &cid.to_string(),
        &serde_json::to_string_pretty(&metadata).expect("Can't print")
    ));

    let item = match serde_json::to_string(metadata) {
        Ok(s) => s,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    if let Err(e) = storage.set_item(&cid.to_string(), &item) {
        ConsoleService::error(&format!("{:?}", e));
    }
}
