//use cid::Cid;
use futures::StreamExt;
use ipfs_api::IpfsClient;
//use multibase::Base;
//use std::convert::TryFrom;
//use std::io::{Read, Write};

/* fn pause() {
    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press enter to exit...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
} */

#[tokio::main]
async fn main() {
    println!("Viewer Application Initializaion...");

    let client = IpfsClient::default();

    let mut stream = client.pubsub_sub("live_like", true);

    while let Some(result) = stream.next().await {
        if let Ok(response) = result {
            println!("{:#?}", response);

            /* let sender = match response.from {
                Some(sender) => sender,
                None => {
                    eprintln!("No Sender");
                    continue;
                }
            }; */

            /* let data = match response.data {
                Some(data) => data,
                None => {
                    eprintln!("No Data");
                    continue;
                }
            }; */

            /* let cid = match Cid::try_from(data.as_str()) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Can't parse CID from string Error: {}", e);
                    continue;
                }
            }; */

            /* let hash = match cid.to_string_of_base(Base::Base58Btc) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Can't parse hash from CID Error: {}", e);
                    continue;
                }
            }; */

            //println!("Sender: {:#?} Message: {:#?}", sender, hash);
        }
    }
}
