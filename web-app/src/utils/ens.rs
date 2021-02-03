use std::convert::TryFrom;
use std::path::PathBuf;

use crate::utils::bindings::ens_get_content_hash;

use yew::services::ConsoleService;
use yew::Callback;

use cid::Cid;

pub async fn get_beacon_from_name(name: String, cb: Callback<Cid>) {
    let js_value = match ens_get_content_hash(&name).await {
        Ok(hash) => hash,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return;
        }
    };

    let path = match js_value.as_string() {
        Some(string) => string,
        None => return,
    };

    let path = PathBuf::try_from(path).expect("Invalid Path");
    let file_name = path.file_name().expect("Invalid File Name");
    let string = file_name.to_str().expect("Invalid Unicode");
    let cid = Cid::try_from(string).expect("Invalid Cid");

    cb.emit(cid);
}
