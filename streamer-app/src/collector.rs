use std::io::Cursor;

use tokio::sync::mpsc::Receiver;

use hyper::body::Bytes;

use ipfs_api::IpfsClient;

use serde::Serialize;

//Hard-coded for now...
const PUBSUB_TOPIC_VIDEO: &str = "livelike/video";

#[derive(Serialize, Debug)]
pub struct DagNode {
    #[serde(rename = "1080p60")]
    latest_1080p60: Option<String>,

    #[serde(rename = "720p60")]
    latest_720p60: Option<String>,

    #[serde(rename = "720p30")]
    latest_720p30: Option<String>,

    #[serde(rename = "480p30")]
    latest_480p30: Option<String>,

    #[serde(rename = "previous")]
    previous: Option<String>,
}

impl Default for DagNode {
    fn default() -> Self {
        Self {
            latest_1080p60: None,
            latest_720p60: None,
            latest_720p30: None,
            latest_480p30: None,
            previous: None,
        }
    }
}

pub enum StreamVariants {
    Stream1080p60,
    Stream720p60,
    Stream720p30,
    Stream480p30,
}

pub async fn collect_video_data(ipfs: IpfsClient, mut rx: Receiver<(StreamVariants, Bytes)>) {
    let mut dag_node: DagNode = DagNode::default();

    while let Some((variant, data)) = rx.recv().await {
        let add = ipfs_api::request::Add {
            trickle: None,
            only_hash: None,
            wrap_with_directory: None,
            chunker: None,
            pin: Some(false),
            raw_leaves: None,
            cid_version: Some(1),
            hash: None,
            inline: None,
            inline_limit: None,
        };

        let cid = match ipfs.add_with_options(Cursor::new(data), add).await {
            Ok(result) => result.hash,
            Err(e) => {
                eprintln!("IPFS add failed {}", e);
                continue;
            }
        };

        #[cfg(debug_assertions)]
        println!("IPFS add => {}", &cid);

        match variant {
            StreamVariants::Stream1080p60 => dag_node.latest_1080p60 = Some(cid),
            StreamVariants::Stream720p60 => dag_node.latest_720p60 = Some(cid),
            StreamVariants::Stream720p30 => dag_node.latest_720p30 = Some(cid),
            StreamVariants::Stream480p30 => dag_node.latest_480p30 = Some(cid),
        }

        if dag_node.latest_480p30.is_none()
            || dag_node.latest_720p30.is_none()
            || dag_node.latest_720p60.is_none()
            || dag_node.latest_1080p60.is_none()
        {
            continue;
        }

        #[cfg(debug_assertions)]
        println!("{:#?}", dag_node);

        let json_string = serde_json::to_string(&dag_node).expect("Can't serialize dag node");

        let cid = match ipfs.dag_put(Cursor::new(json_string)).await {
            Ok(response) => response.cid.cid_string,
            Err(e) => {
                eprintln!("IPFS dag put failed {}", e);
                continue;
            }
        };

        #[cfg(debug_assertions)]
        println!("DAG node CID => {}", &cid);

        if let Err(e) = ipfs.pubsub_pub(PUBSUB_TOPIC_VIDEO, &cid).await {
            eprintln!("IPFS pubsub pub failed {}", e);
            continue;
        }

        println!("GossipSub publish => {}", &cid);

        dag_node.latest_1080p60 = None;
        dag_node.latest_720p60 = None;
        dag_node.latest_720p30 = None;
        dag_node.latest_480p30 = None;

        dag_node.previous = Some(cid);
    }
}
