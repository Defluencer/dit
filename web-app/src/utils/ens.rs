use std::convert::TryFrom;

use wasm_bindgen::JsValue;

use web3::transports::eip_1193::{Eip1193, Provider};
use web3::Web3;

use yew::services::ConsoleService;
use yew::Callback;

use cid::Cid;

#[derive(Clone)]
pub struct EthereumNameService {
    client: Web3<Eip1193>,
}

impl EthereumNameService {
    pub fn new() -> Result<Self, JsValue> {
        let provider = Provider::default()?;

        let transport = Eip1193::new(provider);

        let client = Web3::new(transport);

        Ok(Self { client })
    }

    pub async fn get_beacon_cid(&self, name: String) -> Result<Cid, ()> {
        let name = &format!("defluencer.{}.eth", name);

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("ENS get => {}", name));

        let hash = match self.client.ens().get_content_hash(name).await {
            Ok(hash) => hash,
            Err(e) => {
                ConsoleService::error(&format!("{:#?}", e));
                return Err(());
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Hash => {:#x?}", &hash));

        // https://eips.ethereum.org/EIPS/eip-1577

        if hash.first().is_none() {
            return Err(());
        }

        if &0xe3 != hash.first().unwrap() {
            //Not IPFS
            return Err(());
        }

        let cid = Cid::try_from(&hash[2..]).expect("Invalid Cid");

        Ok(cid)
    }
}

pub async fn get_beacon_from_name(
    client: EthereumNameService,
    name: String,
    cb: Callback<Result<Cid, ()>>,
) {
    cb.emit(client.get_beacon_cid(name).await);
}
