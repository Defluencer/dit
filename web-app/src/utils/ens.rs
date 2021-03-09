use wasm_bindgen::JsValue;

use web3::transports::eip_1193::{Eip1193, Provider};
use web3::Web3;

use yew::services::ConsoleService;
use yew::Callback;

use cid::multihash::MultihashGeneric;
use cid::Cid;
use cid::Version;

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

    pub async fn get_content_cid(&self, name: String) -> Result<Cid, ()> {
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
        ConsoleService::info(&format!("hash => {:#x?}", &hash));

        // https://eips.ethereum.org/EIPS/eip-1577

        if &0xe3 != hash.get(0).expect("Empty Hash") {
            return Err(());
        }

        let version = match hash.get(1).expect("Empty Hash") {
            0 => Version::V0,
            1 => Version::V1,
            _ => return Err(()),
        };

        let content_type = *hash.get(3).expect("Empty Hash") as u64;

        let slice = &hash[4..]; // ignore first 4 bytes

        let multihash = MultihashGeneric::from_bytes(slice).expect("Invalid Multihash");

        let cid = Cid::new(version, content_type, multihash).expect("Invalid Cid");

        Ok(cid)
    }
}

pub async fn get_beacon(client: EthereumNameService, name: String, cb: Callback<Result<Cid, ()>>) {
    cb.emit(client.get_content_cid(name).await);
}
