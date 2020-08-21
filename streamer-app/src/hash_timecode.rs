use std::io::Cursor;

use tokio::sync::mpsc::Receiver;

use serde::Serialize;

use ipfs_api::IpfsClient;

pub struct HashTimecode {
    ipfs: IpfsClient,
    rx: Receiver<Timecode>,

    pub seconds_node: SecondsNode,
    pub minutes_node: MinutesNode,
    pub hours_node: HoursNode,
}

pub enum Timecode {
    Add(String),
    Finalize,
}

impl HashTimecode {
    pub fn new(ipfs: IpfsClient, rx: Receiver<Timecode>) -> Self {
        Self {
            ipfs,
            rx,

            seconds_node: SecondsNode {
                seconds: Vec::with_capacity(60),
            },
            minutes_node: MinutesNode {
                minutes: Vec::with_capacity(60),
            },
            hours_node: HoursNode {
                hours: Vec::with_capacity(24),
            },
        }
    }

    pub async fn collect(&mut self) {
        while let Some(event) = self.rx.recv().await {
            match event {
                Timecode::Add(cid) => self.add_segment_cid(cid).await,
                Timecode::Finalize => self.finalize().await,
            }
        }
    }

    async fn add_segment_cid(&mut self, cid: String) {
        let link = IPLDLink { link: cid };

        self.seconds_node.seconds.push(link);
        //TODO push link number of time equal to video segment duration in seconds

        if self.seconds_node.seconds.len() < 60 {
            return;
        }

        self.collect_seconds().await;

        if self.minutes_node.minutes.len() < 60 {
            return;
        }

        self.collect_minutes().await;
    }

    async fn collect_seconds(&mut self) {
        #[cfg(debug_assertions)]
        println!("{:#?}", &self.seconds_node);

        let json_string =
            serde_json::to_string(&self.seconds_node).expect("Can't serialize seconds node");

        let cid = match self.ipfs.dag_put(Cursor::new(json_string)).await {
            Ok(response) => response.cid.cid_string,
            Err(e) => {
                eprintln!("IPFS dag put failed {}", e);
                return;
            }
        };

        //let cid = Cid::from_str(&cid).expect("CID from dag put response failed");

        self.seconds_node.seconds.clear();

        let link = IPLDLink { link: cid };

        self.minutes_node.minutes.push(link);
    }

    async fn collect_minutes(&mut self) {
        #[cfg(debug_assertions)]
        println!("{:#?}", &self.minutes_node);

        let json_string =
            serde_json::to_string(&self.minutes_node).expect("Can't serialize minutes node");

        let cid = match self.ipfs.dag_put(Cursor::new(json_string)).await {
            Ok(response) => response.cid.cid_string,
            Err(e) => {
                eprintln!("IPFS dag put failed {}", e);
                return;
            }
        };

        //let cid = Cid::from_str(&cid).expect("CID from dag put response failed");

        self.minutes_node.minutes.clear();

        let link = IPLDLink { link: cid };

        self.hours_node.hours.push(link);
    }

    async fn finalize(&self) {
        #[cfg(debug_assertions)]
        println!("{:#?}", &self.hours_node);

        let json_string =
            serde_json::to_string(&self.hours_node).expect("Can't serialize minutes node");

        let hours_node_cid = match self.ipfs.dag_put(Cursor::new(json_string)).await {
            Ok(response) => response.cid.cid_string,
            Err(e) => {
                eprintln!("IPFS dag put failed {}", e);
                return;
            }
        };

        let stream = Stream {
            timecode: IPLDLink {
                link: hours_node_cid,
            },
        };

        #[cfg(debug_assertions)]
        println!("{:#?}", &stream);

        let json_string = serde_json::to_string(&stream).expect("Can't serialize minutes node");

        let stream_cid = match self.ipfs.dag_put(Cursor::new(json_string)).await {
            Ok(response) => response.cid.cid_string,
            Err(e) => {
                eprintln!("IPFS dag put failed {}", e);
                return;
            }
        };

        println!("Stream CID => {}", &stream_cid);
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct IPLDLink {
    #[serde(rename = "/")]
    pub link: String, //TODO find a way to serialize Cid instead of String
}

#[derive(Debug, Serialize)]
pub struct Stream {
    pub timecode: IPLDLink, // ../<StreamHash>/timecode/hours/0/minutes/36/seconds/12/..
}

#[derive(Debug, Serialize)]
pub struct HoursNode {
    pub hours: Vec<IPLDLink>,
}

#[derive(Serialize, Debug)]
pub struct MinutesNode {
    pub minutes: Vec<IPLDLink>,
}

#[derive(Serialize, Debug)]
pub struct SecondsNode {
    pub seconds: Vec<IPLDLink>,
}
