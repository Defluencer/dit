use std::io::Cursor;
use std::sync::mpsc::Receiver;

use tokio::fs::File;

use notify::{op::RENAME, RawEvent};

use ipfs_api::IpfsClient;

use serde::Serialize;

//Hard-coded for now...
const PUBSUB_TOPIC_VIDEO: &str = "livelike/video";

#[derive(Serialize, Debug)]
struct DagNode {
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

pub async fn start(rx: Receiver<RawEvent>, client: IpfsClient) {
    println!("File Watcher Starting... Do not change .ts files while streaming!");

    let mut dag_node: DagNode = DagNode {
        latest_1080p60: None,
        latest_720p60: None,
        latest_720p30: None,
        latest_480p30: None,

        previous: None,
    };

    while let Ok(event) = rx.recv() {
        let op = match event.op {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Watcher Op error. {}", e);
                continue;
            }
        };

        //Files are written to .tmp then renamed to .ts
        if op != RENAME {
            continue;
        }

        let path = match event.path {
            Some(result) => result,
            None => {
                eprintln!("Event path not found");
                continue;
            }
        };

        //Ignore all except .ts files
        if path.extension() == None || path.extension().unwrap() != "ts" {
            continue;
        }

        let path_str = path.to_str().expect("Path invalid UTF-8");

        #[cfg(debug_assertions)]
        println!("Path => {:#?}", path_str);

        let file = match File::open(path_str).await {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Opening file failed {}", e);
                continue;
            }
        };

        let file = file.into_std().await;

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

        let video_segment_cid = match client.add_with_options(file, add).await {
            Ok(result) => result.hash,
            Err(e) => {
                eprintln!("IPFS add failed {}", e);
                continue;
            }
        };

        #[cfg(debug_assertions)]
        println!("IPFS Add => {:#?}", &video_segment_cid);

        let parent = match path.parent() {
            Some(result) => result,
            None => {
                eprintln!("Orphan path. Fix folder structure!");
                continue;
            }
        };

        //TODO use match???
        if parent.ends_with("1080p60") {
            dag_node.latest_1080p60 = Some(video_segment_cid);
        } else if parent.ends_with("720p60") {
            dag_node.latest_720p60 = Some(video_segment_cid);
        } else if parent.ends_with("720p30") {
            dag_node.latest_720p30 = Some(video_segment_cid);
        } else if parent.ends_with("480p30") {
            dag_node.latest_480p30 = Some(video_segment_cid);
        } else {
            eprintln!("Can't deduce segment quality from path. Fix folder structure!");
            continue;
        };

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

        let dag_node_cid = match client.dag_put(Cursor::new(json_string)).await {
            Ok(response) => response.cid.cid_string,
            Err(e) => {
                eprintln!("Adding dag node failed {}", e);
                continue;
            }
        };

        #[cfg(debug_assertions)]
        println!("Dag Node Cid => {}", dag_node_cid);

        if let Err(e) = client.pubsub_pub(PUBSUB_TOPIC_VIDEO, &dag_node_cid).await {
            eprintln!("Can't publish message {}", e);
            continue;
        }

        println!("GossipSub Publish => {}", dag_node_cid);

        dag_node.latest_1080p60 = None;
        dag_node.latest_720p60 = None;
        dag_node.latest_720p30 = None;
        dag_node.latest_480p30 = None;

        dag_node.previous = Some(dag_node_cid);
    }
}

#[cfg(test)]
mod tests {
    //use super::*;
    use std::path::Path;

    #[test]
    fn path_parent() {
        let string = "D:/Videos/Live-Like/1080p60/1920x1080_60_0.ts";
        let path = Path::new(string);

        let folder = Path::new("1080p60");

        let result = path.parent().unwrap().ends_with(folder);

        assert_eq!(result, true);
    }
}
