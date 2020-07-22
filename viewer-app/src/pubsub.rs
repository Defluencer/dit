use std::str;
use std::sync::{Arc, RwLock};

use ipfs_api::response::PubsubSubResponse;
use ipfs_api::IpfsClient;

use multibase::Base;

use tokio::process::Command;
use tokio::stream::StreamExt;

use m3u8_rs::playlist::{MediaPlaylist, MediaSegment};

use serde::Deserialize;

use crate::playlist::Playlists;

const PUBSUB_TOPIC_VIDEO: &str = "livelike/video";

//TODO create config common to both streamer and viewer apps
#[derive(Deserialize)]
struct DagNode {
    #[serde(rename = "1080_60")]
    latest_1080_60: String,

    #[serde(rename = "720_60")]
    latest_720_60: String,

    #[serde(rename = "720_30")]
    latest_720_30: String,

    #[serde(rename = "480_30")]
    latest_480_30: String,

    #[serde(rename = "previous")]
    previous: Option<String>,
}

pub async fn pubsub_sub(playlists: Arc<RwLock<Playlists>>) {
    let client = IpfsClient::default();

    let mut stream = client.pubsub_sub(PUBSUB_TOPIC_VIDEO, true);

    println!("Now listening on topic => {}", PUBSUB_TOPIC_VIDEO);

    //previously received dag node cid
    let mut previous_cid = None;

    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => process_response(&mut previous_cid, &response, &playlists).await,
            Err(error) => {
                eprintln!("{}", error);
                continue;
            }
        }
    }
}

async fn process_response(
    previous_cid: &mut Option<String>,
    response: &PubsubSubResponse,
    playlists: &Arc<RwLock<Playlists>>,
) {
    #[cfg(debug_assertions)]
    println!("Message => {:#?}", response);

    if !is_verified_sender(&response) {
        eprintln!("Unauthorized sender");
        return;
    }

    let cid_v1 = match decode_message(&response) {
        Some(data) => data,
        None => {
            eprintln!("Message with no data");
            return;
        }
    };

    #[cfg(debug_assertions)]
    println!("CID => {}", &cid_v1);

    let dag_node = match get_dag_node(&cid_v1).await {
        Ok(data) => data,
        Err(error) => {
            eprintln!("ipfs dag get. {}", error);
            return;
        }
    };

    if previous_cid.as_ref() != dag_node.previous.as_ref() {
        #[cfg(debug_assertions)]
        println!(
            "Missed an update, previous should be => {}",
            dag_node.previous.as_ref().unwrap()
        );

        //TODO missed a message -> rebuild playlists
    }

    update_playlists(&dag_node, &playlists);

    *previous_cid = Some(cid_v1);
}

fn is_verified_sender(_response: &PubsubSubResponse) -> bool {
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

    true
}

fn decode_message(response: &PubsubSubResponse) -> Option<String> {
    let encoded = response.data.as_ref()?;

    let decoded = Base::decode(&Base::Base64Pad, encoded).expect("Can't decode data");

    Some(String::from_utf8(decoded).expect("Invalid UTF-8"))
}

async fn get_dag_node(cid_v1: &str) -> std::io::Result<DagNode> {
    let output = Command::new("ipfs")
        .args(&["dag", "get", cid_v1])
        .output()
        .await?;

    let json = {
        let mut string = String::from_utf8(output.stdout).expect("Invalid UTF-8");

        string.pop(); //remove last char, a null termination

        string
    };

    let result: DagNode = serde_json::from_str(&json).expect("Can't deserialize dag node");

    Ok(result)
}

fn update_playlists(dag_node: &DagNode, playlists: &Arc<RwLock<Playlists>>) {
    let mut playlists = playlists.write().expect("Lock Poisoned");

    update_playlist(&dag_node.latest_1080_60, &mut playlists.playlist_1080_60);
    update_playlist(&dag_node.latest_720_60, &mut playlists.playlist_720_60);
    update_playlist(&dag_node.latest_720_30, &mut playlists.playlist_720_30);
    update_playlist(&dag_node.latest_480_30, &mut playlists.playlist_480_30);
}

fn update_playlist(cid: &str, playlist: &mut MediaPlaylist) {
    let segment = MediaSegment {
        uri: format!("http://{}.ipfs.localhost:8080", cid),
        duration: 4.0,
        title: None,
        byte_range: None,
        discontinuity: false,
        key: None,
        map: None,
        program_date_time: None,
        daterange: None,
    };

    //5 is hls_list_size
    if playlist.segments.len() >= 5 {
        playlist.segments.remove(0);

        playlist.media_sequence += 1;
    }

    playlist.segments.push(segment);
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

    use ipfs_api::IpfsClient;
    use tokio::runtime::Runtime;

    #[test]
    fn dag_get() {
        let input = "bafyreig67d575ald2neuzdoqjlxjnesvqsbdujv5fwvn6dvere3uaf26ju";

        let client = IpfsClient::default();

        let mut rt = Runtime::new().unwrap();

        let out = rt.block_on(client.dag_get(input));

        println!("{:#?}", out)
    }
}
