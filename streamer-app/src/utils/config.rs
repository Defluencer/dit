use tokio::fs;

use linked_data::config::Configuration;

pub async fn get_config() -> Configuration {
    match fs::read("config.json").await {
        Ok(data) => {
            serde_json::from_slice::<Configuration>(&data).expect("Config serialization failed")
        }
        Err(_) => {
            let config = Configuration::default();

            set_config(&config).await;

            config
        }
    }
}

pub async fn set_config(config: &Configuration) {
    let data = match serde_json::to_vec_pretty(config) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("{:#?}", e);
            return;
        }
    };

    if let Err(e) = fs::write("config.json", data).await {
        eprintln!("{:#?}", e);
    }
}
