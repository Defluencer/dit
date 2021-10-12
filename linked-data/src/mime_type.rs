use crate::IPLDLink;

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use cid::multibase::Base;
use cid::Cid;

#[derive(Serialize, Deserialize)]
pub struct MimeTyped {
    pub mime_type: String,
    pub data: IPLDLink,
}

impl MimeTyped {
    pub fn new<U>(mime_type: U, cid: Cid) -> Self
    where
        U: Into<Cow<'static, str>>,
    {
        Self {
            mime_type: mime_type.into().into_owned(),
            data: cid.into(),
        }
    }

    pub fn data_url(&self, data: &[u8]) -> String {
        format!(
            "data:{};base64,{}",
            self.mime_type,
            Base::Base64.encode(data)
        )
    }

    //TODO once you have a unified IPFS API you could do this
    /* pub async fn data_url(self, ipfs: &IpfsClient ) -> String {
        let data = ipfs.cat(self.data.link).await;
        let data = Base::Base64.encode(data);

        format!("data:{};base64,{}", self.mime_type, data)
    } */
}
