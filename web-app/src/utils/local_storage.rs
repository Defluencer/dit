use std::convert::TryFrom;

use web_sys::{Storage, Window};

use yew::services::ConsoleService;

use cid::Cid;

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

pub fn get_cid(key: &str, storage: Option<&Storage>) -> Option<Cid> {
    let storage = storage?;

    let cid = match storage.get_item(key) {
        Ok(option) => option,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return None;
        }
    };

    let cid = cid?;

    let cid = Cid::try_from(cid).expect("Invalid Cid");

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!("Storage Get => {} \n {}", key, &cid.to_string()));

    Some(cid)
}

pub fn set_cid(key: &str, cid: &Cid, storage: Option<&Storage>) {
    let storage = match storage {
        Some(st) => st,
        None => return,
    };

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!("Storage Set => {} \n {}", key, &cid.to_string()));

    if let Err(e) = storage.set_item(key, &cid.to_string()) {
        ConsoleService::error(&format!("{:#?}", e));
    }
}

pub fn set_local_beacon(ens_name: &str, cid: &Cid, storage: Option<&Storage>) {
    let storage = match storage {
        Some(st) => st,
        None => return,
    };

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!(
        "Storage Set => {} \n {}",
        ens_name,
        &cid.to_string()
    ));

    if let Err(e) = storage.set_item(ens_name, &cid.to_string()) {
        ConsoleService::error(&format!("{:#?}", e));
    }

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!(
        "Storage Set => {} \n {}",
        &cid.to_string(),
        ens_name
    ));

    if let Err(e) = storage.set_item(&cid.to_string(), ens_name) {
        ConsoleService::error(&format!("{:#?}", e));
    }
}

const IPFS_API_ADDRS_KEY: &str = "ipfs_api_addrs";

pub fn set_local_ipfs_addrs(addrs: &str, storage: Option<&Storage>) {
    let storage = match storage {
        Some(st) => st,
        None => return,
    };

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!(
        "Storage Set => {} \n {}",
        IPFS_API_ADDRS_KEY, addrs
    ));

    if let Err(e) = storage.set_item(IPFS_API_ADDRS_KEY, addrs) {
        ConsoleService::error(&format!("{:#?}", e));
    }
}

pub fn get_local_ipfs_addrs(storage: Option<&Storage>) -> Option<String> {
    let storage = storage?;

    let addrs = match storage.get_item(IPFS_API_ADDRS_KEY) {
        Ok(option) => option,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return None;
        }
    };

    let addrs = addrs?;

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!(
        "Storage Get => {} \n {}",
        IPFS_API_ADDRS_KEY, &addrs
    ));

    Some(addrs)
}
