use std::convert::TryFrom;

use web_sys::Storage;

use yew::services::ConsoleService;

use cid::Cid;

const IPFS_API_ADDRS_KEY: &str = "ipfs_api_addrs";

#[derive(Clone)]
pub struct LocalStorage {
    storage: Storage,
}

impl LocalStorage {
    pub fn new() -> Self {
        let window = match web_sys::window() {
            Some(window) => window,
            None => {
                ConsoleService::error("Cannot Access Window Object Aborting...");
                std::process::abort();
            }
        };

        let storage = match window.local_storage() {
            Ok(store) => store,
            Err(e) => {
                ConsoleService::error(&format!("{:#?}", e));
                std::process::abort();
            }
        };

        let storage = match storage {
            Some(storage) => storage,
            None => {
                ConsoleService::error("No Local Storage Object Aborting...");
                std::process::abort();
            }
        };

        Self { storage }
    }

    /* pub fn remove_item(&self, key: &str) {
        if let Err(e) = self.storage.remove_item(key) {
            ConsoleService::error(&format!("{:#?}", e));
        }
    } */

    pub fn get_cid(&self, key: &str) -> Option<Cid> {
        let cid = match self.storage.get_item(key) {
            Ok(option) => option?,
            Err(e) => {
                ConsoleService::error(&format!("{:#?}", e));
                return None;
            }
        };

        let cid = match Cid::try_from(cid) {
            Ok(cid) => cid,
            Err(e) => {
                ConsoleService::error(&format!("{:#?}", e));
                return None;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Storage Get => {} \n {:?}", key, cid));

        Some(cid)
    }

    pub fn set_cid(&self, key: &str, cid: &Cid) {
        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Storage Set => {} \n {:?}", key, cid));

        if let Err(e) = self.storage.set_item(key, &cid.to_string()) {
            ConsoleService::error(&format!("{:#?}", e));
        }
    }

    pub fn set_local_ipfs_addrs(&self, addrs: &str) {
        #[cfg(debug_assertions)]
        ConsoleService::info(&format!(
            "Storage Set => {} \n {}",
            IPFS_API_ADDRS_KEY, addrs
        ));

        if let Err(e) = self.storage.set_item(IPFS_API_ADDRS_KEY, addrs) {
            ConsoleService::error(&format!("{:#?}", e));
        }
    }

    pub fn get_local_ipfs_addrs(&self) -> Option<String> {
        let addrs = match self.storage.get_item(IPFS_API_ADDRS_KEY) {
            Ok(option) => option?,
            Err(e) => {
                ConsoleService::error(&format!("{:#?}", e));
                return None;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!(
            "Storage Get => {} \n {}",
            IPFS_API_ADDRS_KEY, &addrs
        ));

        Some(addrs)
    }
}
