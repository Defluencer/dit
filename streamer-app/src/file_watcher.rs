use std::ffi::OsStr;
use std::sync::mpsc::Receiver;

use tokio::process::Command;

use notify::{op::RENAME, RawEvent};

use ipfs_api::IpfsClient;

const PUBSUB_TOPIC_1080_60: &str = "livelike/video/1080_60";
const PUBSUB_TOPIC_720_60: &str = "livelike/video/720_60";
const PUBSUB_TOPIC_720_30: &str = "livelike/video/720_30";
const PUBSUB_TOPIC_480_30: &str = "livelike/video/480_30";

pub async fn start(rx: Receiver<RawEvent>, client: IpfsClient) {
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

        let output_string = match String::from_utf8(output.stdout) {
            Ok(mut result) => {
                result.pop(); //remove last char, a null termination
                result
            }
            Err(e) => {
                eprintln!("Command Output Invalid UTF8. {}", e);
                continue;
            }
        };

        let cid_v1 = &output_string;

        //TODO create dag node with link to previous and current hash
        //that way the entire video stream is linked together
        //previous_hash = response.hash;

        let parent = match path.parent() {
            Some(result) => result,
            None => {
                eprintln!("Orphan Path");
                continue;
            }
        };

        let topic = if parent.ends_with("1080_60") {
            PUBSUB_TOPIC_1080_60
        } else if parent.ends_with("720_60") {
            PUBSUB_TOPIC_720_60
        } else if parent.ends_with("720_30") {
            PUBSUB_TOPIC_720_30
        } else if parent.ends_with("480_30") {
            PUBSUB_TOPIC_480_30
        } else {
            eprintln!("Can't deduce topic from path.");
            continue;
        };

        if let Err(e) = client.pubsub_pub(topic, cid_v1).await {
            eprintln!("Can't publish message. {}", e);
            continue;
        }

        println!("IPFS Add {:#?} => {:#?}", path, cid_v1);
    }
}

#[cfg(test)]
mod tests {
    //use super::*;
    use std::path::Path;

    #[test]
    fn path_parent() {
        let string = "D:/Videos/Live-Like/1080_60/1920x1080_60_0.ts";
        let path = Path::new(string);

        let folder = Path::new("1080_60");

        let result = path.parent().unwrap().ends_with(folder);

        assert_eq!(result, true);
    }
}
