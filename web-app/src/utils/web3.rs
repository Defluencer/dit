use std::convert::TryFrom;

use web3::transports::eip_1193::{Eip1193, Provider};
use web3::types::Address;
use web3::Web3;

use yew::services::ConsoleService;

use serde::Serialize;

use cid::Cid;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone)]
pub struct Web3Service {
    client: Web3<Eip1193>,
}

impl Web3Service {
    pub fn new() -> Self {
        let provider = match Provider::default() {
            Ok(provider) => provider,
            Err(e) => {
                ConsoleService::error(&format!("{:#?}", e));
                std::process::abort();
            }
        };

        let transport = Eip1193::new(provider);

        let client = Web3::new(transport);

        Self { client }
    }

    pub async fn get_ipfs_content(&self, name: String) -> Result<Cid> {
        let name = &format!("defluencer.{}.eth", name);

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("ENS get => {}", name));

        let hash = self.client.ens().get_content_hash(name).await?;

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Hash => {:x?}", &hash));

        // https://eips.ethereum.org/EIPS/eip-1577

        // IPFS 0xe3, Swarm 0xe4
        if Some(&0xe3) != hash.first() {
            return Err(NotIPFSStorage.into());
        }

        // First 2 bytes are protoCode uvarint
        let cid = Cid::try_from(&hash[2..])?;

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Cid => {}", &cid.to_string()));

        Ok(cid)
    }

    //https://docs.rs/web3/0.15.0/web3/api/struct.Eth.html#method.request_accounts
    pub async fn get_eth_accounts(&self) -> Result<Address> {
        let address = self.client.eth().request_accounts().await?;

        Ok(address[0])
    }

    //https://docs.rs/web3/0.15.0/web3/api/struct.Eth.html#method.sign
    pub async fn eth_sign<T>(&self, addrs: Address, content: T) -> Result<[u8; 65]>
    where
        T: Serialize,
    {
        let data = serde_json::to_vec(&content)?;

        let sign = self.client.personal().sign(addrs, data.into()).await?;

        Ok(sign.to_fixed_bytes())
    }

    //https://eips.ethereum.org/EIPS/eip-181
    pub async fn get_name(&self, addrs: Address) -> Result<String> {
        let res = self.client.ens().get_canonical_name(addrs).await?;

        Ok(res)
    }
}

#[derive(Debug)]
struct NotIPFSStorage;

impl std::fmt::Display for NotIPFSStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Not IPFS storage")
    }
}

impl std::error::Error for NotIPFSStorage {}
