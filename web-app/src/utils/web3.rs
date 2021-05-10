use std::convert::TryFrom;

use wasm_bindgen::JsValue;

use web3::transports::eip_1193::{Eip1193, Provider};
use web3::types::Address;
use web3::{Error, Web3};

use yew::services::ConsoleService;

//use serde::Serialize;

use cid::Cid;

use linked_data::chat::Content;

#[derive(Clone)]
pub struct Web3Service {
    client: Web3<Eip1193>,
}

impl Web3Service {
    pub fn new() -> Result<Self, JsValue> {
        let provider = Provider::default()?;

        let transport = Eip1193::new(provider);

        let client = Web3::new(transport);

        Ok(Self { client })
    }

    pub async fn get_ipfs_content(&self, name: String) -> Result<Cid, web3::contract::Error> {
        let name = &format!("defluencer.{}.eth", name);

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("ENS get => {}", name));

        let hash = self.client.ens().get_content_hash(name).await?;

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Hash => {:x?}", &hash));

        // https://eips.ethereum.org/EIPS/eip-1577

        // IPFS 0xe3, Swarm 0xe4
        if Some(&0xe3) != hash.first() {
            return Err(Error::InvalidResponse("Not IPFS storage".to_owned()).into());
        }

        // First 2 bytes are protoCode uvarint
        let cid = match Cid::try_from(&hash[2..]) {
            Ok(cid) => cid,
            Err(_) => return Err(Error::InvalidResponse("Invalid CID".to_owned()).into()),
        };

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Cid => {}", &cid.to_string()));

        Ok(cid)
    }

    //https://docs.rs/web3/0.15.0/web3/api/struct.Eth.html#method.request_accounts
    pub async fn get_eth_accounts(&self) -> Result<Address, Error> {
        let address = self.client.eth().request_accounts().await?;

        Ok(address[0])
    }

    //https://docs.rs/web3/0.15.0/web3/api/struct.Eth.html#method.sign
    pub async fn eth_sign(&self, addrs: Address, content: Content) -> Result<[u8; 65], Error> {
        let data = serde_json::to_vec(&content).expect("Cannot Serialize");

        let sign = self.client.personal().sign(addrs, data.into()).await?;

        Ok(sign.to_fixed_bytes())
    }

    //https://eips.ethereum.org/EIPS/eip-181
    pub async fn get_name(&self, addrs: Address) -> Result<String, web3::contract::Error> {
        self.client.ens().get_canonical_name(addrs).await
    }
}

/* #[derive(Serialize)]
struct SignTypedData {
    domain: Domain,

    message: Content,

    #[serde(rename = "primaryType")]
    primary_type: String,

    types: Types,
} */

/* impl SignTypedData {
    fn new(addrs: Address, content: Content) -> Self {
        Self {
            domain: Domain {
                chain_id: 3,
                name: "Defluencer".to_owned(),
                verifying_contract: addrs,
                version: "0.1".to_owned(),
            },
            message: content,
            primary_type: "Content".to_owned(),
            types: Types {
                eip_712_domain: vec![
                    DomainType {
                        name: "name".to_owned(),
                        domain_type: "string".to_owned(),
                    },
                    DomainType {
                        name: "version".to_owned(),
                        domain_type: "string".to_owned(),
                    },
                    DomainType {
                        name: "chainId".to_owned(),
                        domain_type: "uint256".to_owned(),
                    },
                    DomainType {
                        name: "verifyingContract".to_owned(),
                        domain_type: "address".to_owned(),
                    },
                ],

                content: vec![
                    DomainType {
                        name: "name".to_owned(),
                        domain_type: "string".to_owned(),
                    },
                    DomainType {
                        name: "peer_id".to_owned(),
                        domain_type: "string".to_owned(),
                    },
                ],
            },
        }
    }
} */

/* #[derive(Serialize)]
struct Domain {
    #[serde(rename = "chainId")]
    chain_id: usize,

    name: String,

    #[serde(rename = "verifyingContract")]
    verifying_contract: Address,

    version: String,
} */

/* #[derive(Serialize)]
struct Types {
    #[serde(rename = "EIP712Domain")]
    eip_712_domain: Vec<DomainType>,

    #[serde(rename = "Content")]
    content: Vec<DomainType>,
} */

/* #[derive(Serialize)]
struct DomainType {
    name: String,

    #[serde(rename = "type")]
    domain_type: String,
} */
