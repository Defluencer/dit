use std::io::Cursor;

use serde::Serialize;

use ipfs_api::IpfsClient;

pub struct HashTimecode {
    ipfs: IpfsClient,

    pub seconds_node: SecondsNode,
    pub minutes_node: MinutesNode,
    pub hours_node: HoursNode,
}

impl HashTimecode {
    pub fn new(ipfs: IpfsClient) -> Self {
        Self {
            ipfs,

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

    pub fn add_segment_cid(&mut self, cid: String) {
        let link = IPLDLink { link: cid };

        self.seconds_node.seconds.push(link);

        if self.seconds_node.seconds.len() < 15 {
            return;
        }

        self.collect_seconds();

        if self.minutes_node.minutes.len() < 60 {
            return;
        }

        self.collect_minutes();
    }

    async fn collect_seconds(&mut self) {
        let json_string =
            serde_json::to_string(&self.seconds_node).expect("Can't serialize dag node");

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
        let json_string =
            serde_json::to_string(&self.minutes_node).expect("Can't serialize dag node");

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

    pub fn finalize(&mut self) {
        //TODO collect latest cids and bundle them.
    }
}

#[derive(Debug, Serialize)]
pub struct IPLDLink {
    #[serde(rename = "/")]
    pub link: String, //TODO find a way to store Cid instead of String
}

#[derive(Debug, Serialize)]
pub struct Stream {
    pub start: IPLDLink,    // ../<StreamHash>/start/..
    pub timecode: IPLDLink, // ../<StreamHash>/timecode/hours/0/minutes/36/seconds/12/..
    pub end: IPLDLink,      // ../<StreamHash>/end/..
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
