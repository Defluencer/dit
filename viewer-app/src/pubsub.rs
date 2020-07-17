use crate::playlist::Playlist;
use cid::Cid;
use ipfs_api::IpfsClient;
use multibase::Base;
use std::str;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tokio::stream::StreamExt;

pub async fn pubsub_sub(topic: &str, playlist: Arc<RwLock<Playlist>>) {
    let client = IpfsClient::default();

    let mut stream = client.pubsub_sub(topic, true);

    println!("Initialization Complete!");

    while let Some(result) = stream.next().await {
        if let Ok(response) = result {
            //println!("{:#?}", response);

            //TODO match sender id VS streamer is
            /* let sender = match response.from {
                Some(sender) => {
                    let decoded = match Base::decode(&Base::Base64Pad, sender) {
                        Ok(result) => result,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            continue;
                        }
                    };

                    match String::from_utf8(decoded) {
                        Ok(result) => result,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            continue;
                        }
                    }
                }
                None => {
                    eprintln!("No Sender");
                    continue;
                }
            }; */

            let encoded = match response.data {
                Some(data) => data,
                None => {
                    eprintln!("No Data");
                    continue;
                }
            };

            let decoded = match Base::decode(&Base::Base64Pad, encoded) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Can't decode data. {}", e);
                    continue;
                }
            };

            let cid_v1_string = match str::from_utf8(&decoded) {
                Ok(cid) => cid,
                Err(e) => {
                    eprintln!("Invalid UTF-8 {}", e);
                    continue;
                }
            };

            println!("CID: {}", cid_v1_string);

            let cid = match Cid::from_str(cid_v1_string) {
                Ok(cid) => cid,
                Err(e) => {
                    eprintln!("Can't get cid from str. {}", e);
                    continue;
                }
            };

            match playlist.write() {
                //Could use tokio async RwLock
                Ok(mut playlist) => {
                    playlist.add_segment(cid);
                }
                Err(e) => {
                    eprintln!("Lock poisoned. {}", e);
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use cid::Cid;
    use multibase::Base;
    use std::str::FromStr;

    #[test]
    fn decode_base64pad() {
        let input = "QmQrj21qtpNyx5hH8YTWMMja3Tuhwd4Y6XUmk3V6UJ5rhD";

        println!("Input Message: {:#?}", input);

        let encoded = Base::encode(&Base::Base64Pad, input);

        println!("Encoded Message: {:#?}", encoded);

        let decoded = Base::decode(&Base::Base64Pad, encoded).expect("Error: ");

        let output = std::str::from_utf8(&decoded).expect("Error: ");

        println!("Output Message: {:#?}", output);

        assert_eq!(input, output);
    }

    #[test]
    fn encode_cids() {
        let input = "QmQrj21qtpNyx5hH8YTWMMja3Tuhwd4Y6XUmk3V6UJ5rhD";

        println!("Input Message: {:#?}", input);

        let encoded = Cid::from_str(input).expect("Can't get cid from str");

        println!("Encoded Message: {:?}", encoded);

        let decoded = encoded.to_string_of_base(Base::Base58Btc).expect("Error: ");

        let output = &decoded;

        println!("Output Message: {:#?}", output);

        assert_eq!(input, output);
    }
}
