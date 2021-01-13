use tokio::stream::StreamExt;

use hyper::body::Bytes;

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;

use linked_data::config::Config;

pub async fn _get_config(ipfs: &IpfsClient) -> Config {
    if let Ok(config) = ipfs.name_resolve(None, false, false).await {
        let buffer: Result<Bytes, Error> = ipfs.dag_get(&config.path).collect().await;

        if let Ok(buffer) = buffer {
            if let Ok(config) = serde_json::from_slice(&buffer) {
                return config;
            }
        }
    }

    let config = Config::default();

    eprintln!("Cannot load config. Fallback -> {:#?}", config);

    config
}
