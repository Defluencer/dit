use std::convert::TryFrom;
use std::str;
use std::sync::{Arc, RwLock};

use cid::Cid;
use ipfs_api::response::{Error, PubsubSubResponse};
use ipfs_api::IpfsClient;
use multibase::Base;

use tokio::stream::StreamExt;

use m3u8_rs::playlist::{MediaPlaylist, MediaSegment};

use serde::Deserialize;

use crate::playlist::{Playlists, HLS_LIST_SIZE};

// Hard-Coded for now...
const PUBSUB_TOPIC_VIDEO: &str = "livelike/video";
const STREAMER_PEER_ID: &str = "QmX91oLTbANP7NV5yUYJvWYaRdtfiaLTELbYVX5bA8A9pi";

#[derive(Deserialize, Clone, Debug)]
struct DagNode {
    #[serde(rename = "1080p60")]
    latest_1080p60: String,

    #[serde(rename = "720p60")]
    latest_720p60: String,

    #[serde(rename = "720p30")]
    latest_720p30: String,

    #[serde(rename = "480p30")]
    latest_480p30: String,

    #[serde(rename = "previous")]
    previous: Option<String>,
}

pub async fn pubsub_sub(playlists: Arc<RwLock<Playlists>>) {
    let client = IpfsClient::default();

    let mut stream = client.pubsub_sub(PUBSUB_TOPIC_VIDEO, true);

    #[cfg(debug_assertions)]
    println!("Now listening to topic => {}", PUBSUB_TOPIC_VIDEO);

    //previously received dag node cid
    let mut previous_cid = None;

    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                process_response(&mut previous_cid, &response, &playlists, &client).await
            }
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
    client: &IpfsClient,
) {
    #[cfg(debug_assertions)]
    println!("Message => {:#?}", response);

    if !is_verified_sender(response) {
        eprintln!("Unauthorized sender");
        return;
    }

    let dag_node_cid = match decode_message(response) {
        Some(data) => data,
        None => {
            eprintln!("Message with no data");
            return;
        }
    };

    println!("Dag Node CID => {}", &dag_node_cid);

    let dag_node = match get_dag_node(client, &dag_node_cid).await {
        Ok(data) => data,
        Err(error) => {
            eprintln!("IPFS dag get failed {}", error);
            return;
        }
    };

    if previous_cid.as_ref() != dag_node.previous.as_ref() {
        println!("Missed an update, rebuilding playlists...");

        rebuild_playlists(dag_node, playlists, previous_cid, client).await;
    } else {
        update_playlists(dag_node, playlists);
    }

    *previous_cid = Some(dag_node_cid);
}

fn is_verified_sender(response: &PubsubSubResponse) -> bool {
    let encoded = match response.from.as_ref() {
        Some(sender) => sender,
        None => return false,
    };

    let decoded = Base::decode(&Base::Base64Pad, encoded).expect("Decoding sender failed");

    let cid = Cid::try_from(decoded).expect("CID from decoded sender failed");

    #[cfg(debug_assertions)]
    println!("Sender => {}", cid.to_string());

    cid.to_string() == STREAMER_PEER_ID
}

fn decode_message(response: &PubsubSubResponse) -> Option<String> {
    let encoded = response.data.as_ref()?;

    let decoded = Base::decode(&Base::Base64Pad, encoded).expect("Decoding message failed");

    let message = String::from_utf8(decoded).expect("Decoded message invalid UTF-8");

    Some(message)
}

async fn get_dag_node(client: &IpfsClient, cid_v1: &str) -> Result<DagNode, Error> {
    let json = client.dag_get(cid_v1).await?;

    let result: DagNode = serde_json::from_str(&json).expect("Deserializing dag node failed");

    Ok(result)
}

///Rebuild playlists by folowing the dag node link chain.
async fn rebuild_playlists(
    latest_dag_node: DagNode,
    playlists: &Arc<RwLock<Playlists>>,
    previous_cid: &Option<String>,
    client: &IpfsClient,
) {
    let mut missing_nodes = Vec::with_capacity(HLS_LIST_SIZE);

    missing_nodes.push(latest_dag_node);

    while missing_nodes.last().unwrap().previous != *previous_cid {
        //Fill the vec with all the missing nodes.

        let dag_node_cid = missing_nodes
            .last()
            .unwrap()
            .previous
            .as_ref()
            .expect("Dag Node previous link empty while having previously received a node.");

        let dag_node = match get_dag_node(client, dag_node_cid).await {
            Ok(data) => data,
            Err(error) => {
                eprintln!("IPFS dag get failed {}", error);
                return;
            }
        };

        missing_nodes.push(dag_node);

        if missing_nodes.last().unwrap().previous == None {
            //Found first node of the stream, stop here.
            break;
        }

        if missing_nodes.len() >= HLS_LIST_SIZE {
            //Found more node than the list size, stop here.
            break;
        }
    }

    for dag_node in missing_nodes.into_iter().rev() {
        #[cfg(debug_assertions)]
        println!("Missing Dag Node => {:#?}", &dag_node);

        update_playlists(dag_node, playlists);
    }
}

///Update playlists with the dag node links.
fn update_playlists(dag_node: DagNode, playlists: &Arc<RwLock<Playlists>>) {
    let mut playlists = playlists.write().expect("Lock poisoned");

    update_playlist(&dag_node.latest_1080p60, &mut playlists.playlist_1080_60);
    update_playlist(&dag_node.latest_720p60, &mut playlists.playlist_720_60);
    update_playlist(&dag_node.latest_720p30, &mut playlists.playlist_720_30);
    update_playlist(&dag_node.latest_480p30, &mut playlists.playlist_480_30);
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

    if playlist.segments.len() >= HLS_LIST_SIZE {
        playlist.segments.remove(0);

        playlist.media_sequence += 1;
    }

    playlist.segments.push(segment);
}

#[cfg(test)]
mod tests {
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

    use cid::Cid;
    use std::convert::TryFrom;

    #[test]
    fn decode_sender() {
        let encoded = "EiCCvg8x8AnKNrYt1bb/816hxWZmzcLKGa33jF5qh9lN5w==";

        println!("Encoded Message: {:#?}", encoded);

        let decoded = Base::decode(&Base::Base64Pad, encoded).expect("decode_sender => ");

        println!("Message Length: {:#?}", decoded.len());

        let cid = Cid::try_from(decoded).expect("decode_sender => ");

        println!("from: {:#?}", cid.to_string());
    }

    #[test]
    fn decode_seqno() {
        let encoded = "FiWcAiAoTRQ=";

        println!("Encoded Message: {:#?}", encoded);

        let decoded = Base::decode(&Base::Base64Pad, encoded).expect("decode_seqno => ");

        println!("Message Length: {:#?}", decoded.len());

        let seqno = {
            //Fugly but I don't care!!!
            let mut array = [0u8; 8];

            for (i, byte) in decoded.into_iter().enumerate() {
                if i > 8 {
                    break;
                }

                array[i] = byte;
            }

            u64::from_ne_bytes(array)
        };

        println!("seqno: {:#?}", seqno);
    }
}
