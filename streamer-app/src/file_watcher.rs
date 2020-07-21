use std::ffi::OsStr;
use std::sync::mpsc::Receiver;

use tokio::process::Command;

use notify::{op::RENAME, RawEvent};

use ipfs_api::IpfsClient;

use serde::Serialize;

const PUBSUB_TOPIC_VIDEO: &str = "live_like_video";

#[derive(Serialize)]
struct DagNode {
    #[serde(rename = "1080_60")]
    latest_1080_60: Option<String>,

    #[serde(rename = "720_60")]
    latest_720_60: Option<String>,

    #[serde(rename = "720_30")]
    latest_720_30: Option<String>,

    #[serde(rename = "480_30")]
    latest_480_30: Option<String>,

    #[serde(rename = "previous")]
    previous: Option<String>,
}

pub async fn start(rx: Receiver<RawEvent>, client: IpfsClient) {
    println!("File Watcher Starting...");
    println!("Do not rename .ts files while streaming");

    let mut dag_node: DagNode = DagNode {
        latest_1080_60: None,
        latest_720_60: None,
        latest_720_30: None,
        latest_480_30: None,

        previous: None,
    };

    while let Ok(event) = rx.recv() {
        let op = match event.op {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Watcher Op Error. {}", e);
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
                eprintln!("Event Path Not Found");
                continue;
            }
        };

        //Ignore .m3u8 files
        if path.extension() != Some(OsStr::new("ts")) {
            continue;
        }

        let path_str = match path.to_str() {
            Some(result) => result,
            None => {
                eprintln!("Path Invalid UTF8");
                continue;
            }
        };

        #[cfg(debug_assertions)]
        println!("Path => {:#?}", path_str);

        let output = match Command::new("ipfs")
            .args(&["add", "-Q", "--pin=false", "--cid-version=1", path_str])
            .output()
            .await
        {
            Ok(result) => result,
            Err(e) => {
                eprintln!("IPFS Add Command Failed. {}", e);
                continue;
            }
        };

        let cid_v1 = match String::from_utf8(output.stdout) {
            Ok(mut result) => {
                result.pop(); //remove last char, a null termination
                result
            }
            Err(e) => {
                eprintln!("Command Output Invalid UTF8. {}", e);
                continue;
            }
        };

        println!("IPFS Add => {:#?}", &cid_v1);

        let parent = match path.parent() {
            Some(result) => result,
            None => {
                eprintln!("Orphan Path");
                continue;
            }
        };

        //TODO use match???
        if parent.ends_with("1080_60") {
            dag_node.latest_1080_60 = Some(cid_v1);
        } else if parent.ends_with("720_60") {
            dag_node.latest_720_60 = Some(cid_v1);
        } else if parent.ends_with("720_30") {
            dag_node.latest_720_30 = Some(cid_v1);
        } else if parent.ends_with("480_30") {
            dag_node.latest_480_30 = Some(cid_v1);
        } else {
            eprintln!("Can't deduce segment quality from path. Fix folder structure");
            continue;
        };

        if dag_node.latest_480_30.is_none()
            || dag_node.latest_720_30.is_none()
            || dag_node.latest_720_60.is_none()
            || dag_node.latest_1080_60.is_none()
        {
            continue;
        }

        let json_string = serde_json::to_string(&dag_node).expect("Can't serialize dag node");

        #[cfg(debug_assertions)]
        println!("Input => {:#?}", json_string);

        //TODO the command accept a file only
        let output = match Command::new("ipfs")
            .args(&["dag", "put", &json_string])
            .output()
            .await
        {
            Ok(result) => result,
            Err(e) => {
                eprintln!("IPFS Dag Put Command Failed. {}", e);
                continue;
            }
        };

        let cid_v1 = match String::from_utf8(output.stdout) {
            Ok(mut result) => {
                result.pop(); //remove last char, a null termination
                result
            }
            Err(e) => {
                eprintln!("Command Output Invalid UTF8. {}", e);
                continue;
            }
        };

        println!("IPFS Dag Put => {}", cid_v1);

        if let Err(e) = client.pubsub_pub(PUBSUB_TOPIC_VIDEO, &cid_v1).await {
            eprintln!("Can't publish message. {}", e);
            continue;
        }

        dag_node.latest_1080_60 = None;
        dag_node.latest_720_60 = None;
        dag_node.latest_720_30 = None;
        dag_node.latest_480_30 = None;

        dag_node.previous = Some(cid_v1);
    }
}

#[cfg(test)]
mod tests {
    //use super::*;
    use std::path::Path;

    #[test]
    fn path_parent() {
        let string = "D:\\Videos\\Live-Like\\1080_60\\1920x1080_60_0.ts";
        let path = Path::new(string);

        let folder = Path::new("1080_60");

        let result = path.parent().unwrap().ends_with(folder);

        assert_eq!(result, true);
    }
}
