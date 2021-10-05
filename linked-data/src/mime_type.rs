use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use cid::multibase::Base;

#[derive(Serialize, Deserialize)]
pub struct MimeTyped {
    mime_type: String,
    data: String,
}

impl MimeTyped {
    pub fn new<U>(mime_type: U, data: &[u8]) -> Self
    where
        U: Into<Cow<'static, str>>,
    {
        Self {
            mime_type: mime_type.into().into_owned(),
            data: Base::Base64.encode(data),
        }
    }

    pub fn data_url(&self) -> String {
        format!("data:{};base64,{}", self.mime_type, self.data)
    }
}
