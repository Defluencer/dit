use crate::utils::bindings::ens_get_content_hash;

use yew::services::ConsoleService;
use yew::Callback;

use cid::multibase::Base;
use cid::multihash::MultihashGeneric;
use cid::Cid;

pub async fn get_beacon_from_name(mut name: String, cb: Callback<Cid>) {
    name.insert_str(0, "defluencer.");

    name.push_str(".eth");

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!("ENS get => {}", &name));

    let js_value = match ens_get_content_hash(&name).await {
        Ok(hash) => hash,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return;
        }
    };

    //btc58 encoded multihash
    let encoded = match js_value.as_string() {
        Some(string) => string,
        None => return,
    };

    let data = Base::decode(&Base::Base58Btc, encoded).expect("Can't decode");

    let hash = MultihashGeneric::from_bytes(&data).expect("Not multihash");

    let cid = Cid::new_v1(0x71, hash);

    cb.emit(cid);
}
