use ipfs_api::IpfsClient;
use notify::{op::RENAME, raw_watcher, RecursiveMode, Watcher};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::mpsc::channel;

fn pause() {
    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press enter to exit...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

#[tokio::main]
async fn main() {
    println!("Init...");

    let client = IpfsClient::default();

    let _res = client.pubsub_sub("live_like", true);

    let (tx, rx) = channel();

    //Raw watcher is used to minimize latency,
    //it work well with ffmpeg option to write a .tmp file first and then
    //rename it .ts when done writing to it.
    let mut watcher = match raw_watcher(tx) {
        Ok(watcher) => {
            println!("Watcher started...");
            watcher
        }
        Err(e) => {
            eprintln!("Can't start watcher Error: {}", e);
            pause();
            return;
        }
    };

    let watch_path = Path::new("./Live-Like/");

    match watcher.watch(watch_path, RecursiveMode::NonRecursive) {
        Ok(_) => println!("Watching {:?}", watch_path),
        Err(e) => {
            eprintln!("Can't watch Error: {}", e);
            pause();
            return;
        }
    }

    //let mut previous_hash = String::new();

    while let Ok(event) = rx.recv() {
        //println!("{:?}", event);

        let path = match event.path {
            Some(path) => path,
            None => continue,
        };

        match event.op {
            Ok(op) => {
                if op != RENAME {
                    continue;
                }
            }
            Err(_) => continue,
        }

        if path.extension() != Some(OsStr::new("ts")) {
            continue;
        }

        let file = match File::open(&path) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Can't open file Error: {}", e);
                pause();
                return;
            }
        };

        let response = match client.add(file).await {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Can't add file Error: {}", e);
                pause();
                return;
            }
        };

        //TODO create dag node with link to previous and current hash
        //that way the entire stream is linked together

        //previous_hash = response.hash;

        println!("Path: {:?} Hash: {}", &path, response.hash);

        if let Err(e) = client.pubsub_pub("live_like", &response.hash).await {
            eprintln!("Can't publish a message Error: {}", e);
            pause();
            return;
        }
    }
}
