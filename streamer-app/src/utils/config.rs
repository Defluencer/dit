use tokio::fs;

use linked_data::config::Configuration;

pub async fn get_config() -> Configuration {
    match fs::read("config.json").await {
        Ok(data) => {
            serde_json::from_slice::<Configuration>(&data).expect("Config serialization failed")
        }
        Err(_) => {
            let config = Configuration::default();

            println!("No configuration file detected. Default {:#?}", config);

            match serde_json::to_vec_pretty(&config) {
                Ok(data) => {
                    if fs::write("config.json", data).await.is_err() {
                        eprintln!("Cannot write config to disk");
                    }
                }
                Err(e) => eprintln!("{:#?}", e),
            }

            config
        }
    }
}
