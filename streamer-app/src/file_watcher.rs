use ipfs_api::IpfsClient;
use notify::{op::RENAME, raw_watcher, RecursiveMode, Watcher};
use std::ffi::OsStr;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;

const LOCAL_FOLDER: &str = "./";

fn pause() {
    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press enter to exit...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

async fn file_watch() {
    let (tx, rx) = channel();

    //Raw watcher is used to minimize latency,
    //it work well with ffmpeg option to write a .tmp file first then
    //rename it when done writing.
    let mut watcher = match raw_watcher(tx) {
        Ok(watcher) => watcher,
        Err(e) => {
            eprintln!("Can't start file watcher {}", e);
            pause();
            return;
        }
    };

    let watch_path = Path::new(LOCAL_FOLDER);

    if let Err(e) = watcher.watch(watch_path, RecursiveMode::NonRecursive) {
        eprintln!("Can't watch local folder {}", e);
        pause();
        return;
    }

    //println!("Initialization Complete!");

    while let Ok(event) = rx.recv() {
        match event.op {
            Ok(op) => {
                if op != RENAME {
                    continue;
                }
            }
            Err(_) => continue,
        }

        let path = match event.path {
            Some(path) => path,
            None => continue,
        };

        if path.extension() != Some(OsStr::new("ts")) {
            continue;
        }

        let file_name = match path.file_name() {
            Some(result) => match result.to_str() {
                Some(name) => name,
                None => {
                    eprintln!("Can't form file name str from OsStr");
                    continue;
                }
            },
            None => {
                eprintln!("Can't get file name from path");
                continue;
            }
        };

        let output = match Command::new("ipfs")
            .args(&["add", "-Q", "--pin=false", "--cid-version=1", file_name])
            .output()
        {
            Ok(result) => result,
            Err(e) => {
                eprintln!("ipfs add command failed. {}", e);
                continue;
            }
        };

        let mut output_string = String::from_utf8(output.stdout).expect("Invalid UTF8");
        output_string.pop(); //remove last char, a null termination

        let cid_v1 = &output_string;

        //TODO create dag node with link to previous and current hash
        //that way the entire video stream is linked together
        //previous_hash = response.hash;

        if let Err(e) = client.pubsub_pub(PUBSUB_TOPIC_VIDEO, cid_v1).await {
            eprintln!("Can't publish message. {}", e);
            continue;
        }

        println!("File: {:#?} CID: {:#?}", file_name, cid_v1);
    }
}
