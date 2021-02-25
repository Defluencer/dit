use wasm_bindgen::JsValue;

use web3::transports::eip_1193::{Eip1193, Provider};
use web3::Web3;

use yew::services::ConsoleService;
use yew::Callback;

use cid::multibase::Base;
use cid::multihash::MultihashGeneric;
use cid::Cid;

pub struct EthereaumNameService {
    client: Web3<Eip1193>,
}

impl EthereaumNameService {
    pub fn new() -> Result<Self, JsValue> {
        let provider = Provider::default()?;

        let transport = Eip1193::new(provider);

        let client = Web3::new(transport);

        Ok(Self { client })
    }
}

pub async fn get_beacon_from_name(mut name: String, cb: Callback<Result<Cid, ()>>) {
    let client = EthereaumNameService::new().unwrap().client;

    let res = client.eth().request_accounts().await;

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!("ENS get => {:#?}", &res));

    /* name.insert_str(0, "defluencer.");

    name.push_str(".eth");

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!("ENS get => {}", &name));

    let js_value = match ens_get_content_hash(&name).await {
        Ok(hash) => hash,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));

            cb.emit(Err(()));
            return;
        }
    };

    //btc58 encoded multihash
    let encoded = match js_value.as_string() {
        Some(string) => string,
        None => {
            cb.emit(Err(()));
            return;
        }
    };

    let data = Base::decode(&Base::Base58Btc, encoded).expect("Can't decode");

    let hash = MultihashGeneric::from_bytes(&data).expect("Not multihash");

    let cid = Cid::new_v1(0x71, hash);

    cb.emit(Ok(cid)); */

    cb.emit(Err(()));
}
