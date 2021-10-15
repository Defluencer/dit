use std::convert::TryFrom;

use crate::cli::content::{COMMENTS_KEY, FEED_KEY};
use crate::cli::friends::FRIENDS_KEY;
use crate::cli::identity::IDENTITY_KEY;
use crate::cli::live::LIVE_KEY;
use crate::cli::moderation::{BANS_KEY, MODS_KEY};
use crate::utils::config::Configuration;
use crate::utils::dag_nodes::{
    ipfs_dag_get_node_async, ipfs_dag_put_node_async, search_keypairs, update_ipns,
};

use tokio::task::JoinHandle;

use serde::Serialize;

use ipfs_api::response::{Error, PinAddResponse, PinRmResponse};
use ipfs_api::IpfsClient;
use ipfs_api::KeyType;

use linked_data::beacon::Beacon;
use linked_data::comments::Commentary;
use linked_data::feed::FeedAnchor;
use linked_data::friends::Friendlies;
use linked_data::identity::Identity;
use linked_data::keccak256;
use linked_data::live::Live;
use linked_data::moderation::{Bans, Moderators};

use structopt::StructOpt;

use cid::multibase::{encode, Base};
use cid::Cid;

#[derive(Debug, StructOpt)]
pub struct BeaconCLI {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Create a new beacon.
    Create(Create),

    /// Pin a beacon.
    /// Will recursively pin all associated data.
    /// The amount of data to be pinned could be MASSIVE use carefully.
    Pin(Pin),

    /// Unpin a beacon.
    /// Recursively unpin all associated data.
    Unpin(Unpin),
}

pub async fn beacon_cli(cli: BeaconCLI) {
    let res = match cli.cmd {
        Command::Create(create) => create_beacon(create).await,
        Command::Pin(pin) => pin_beacon(pin).await,
        Command::Unpin(unpin) => unpin_beacon(unpin).await,
    };

    if let Err(e) = res {
        eprintln!("❗ IPFS: {:#?}", e);
    }
}

#[derive(Debug, StructOpt)]
pub struct Create {
    /// Your choosen display name.
    #[structopt(short, long)]
    display_name: String,

    /// Link to an image avatar.
    #[structopt(short, long)]
    avatar: Cid,
}

async fn create_beacon(args: Create) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    let Create {
        display_name,
        avatar,
    } = args;

    println!("Creating Beacon...");

    let mut config = match Configuration::from_file().await {
        Ok(conf) => conf,
        Err(e) => {
            eprintln!("❗ Cannot get configuration file. Error: {:#?}", e);
            eprintln!("Using Default...");
            Configuration::default()
        }
    };

    config.chat.topic = encode(
        Base::Base32Lower,
        &keccak256(&format!("{}_video", &display_name).into_bytes()),
    );
    config.video.pubsub_topic = encode(
        Base::Base32Lower,
        &keccak256(&format!("{}_chat", &display_name).into_bytes()),
    );

    config.save_to_file().await?;

    let res = ipfs.id(None).await?;
    let peer_id = res.id;

    #[cfg(debug_assertions)]
    println!("IPFS: peer id => {}", &peer_id);

    let live = Live {
        video_topic: config.video.pubsub_topic,
        chat_topic: config.chat.topic,
        peer_id,
    };

    let identity = Identity {
        display_name,
        avatar: avatar.into(),
    };

    let key_list = ipfs.key_list().await?;

    let (identity, content_feed, comments, live, friends, bans, mods) = tokio::try_join!(
        create_ipns_link::<Identity>(&ipfs, "Identity", IDENTITY_KEY, &key_list, Some(identity)),
        create_ipns_link::<FeedAnchor>(&ipfs, "Content Feed", FEED_KEY, &key_list, None),
        create_ipns_link::<Commentary>(&ipfs, "Comments", COMMENTS_KEY, &key_list, None),
        create_ipns_link::<Live>(&ipfs, "Live", LIVE_KEY, &key_list, Some(live)),
        create_ipns_link::<Friendlies>(&ipfs, "Friends", FRIENDS_KEY, &key_list, None),
        create_ipns_link::<Bans>(&ipfs, "Bans", BANS_KEY, &key_list, None),
        create_ipns_link::<Moderators>(&ipfs, "Mods", MODS_KEY, &key_list, None),
    )?;

    let beacon = linked_data::beacon::Beacon {
        identity,
        content_feed: Some(content_feed),
        comments: Some(comments),
        friends: Some(friends),
        live: Some(live),
        bans: Some(bans),
        mods: Some(mods),
    };

    let cid = ipfs_dag_put_node_async(&ipfs, &beacon).await?;

    if let Err(e) = ipfs.pin_add(&cid.to_string(), false).await {
        eprintln!("❗ IPFS could not pin {}. Error: {}", cid.to_string(), e);
    }

    println!("✅ Created Beacon {}", &cid);

    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct Pin {
    /// Beacon CID.
    #[structopt(short, long)]
    cid: Cid,
}

async fn pin_beacon(args: Pin) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    let Pin { cid } = args;

    println!("Getting Beacon...");

    let beacon = ipfs_dag_get_node_async(&ipfs, &cid.to_string()).await?;

    let Beacon {
        identity,
        content_feed,
        comments,
        friends,
        live,
        bans,
        mods,
    } = beacon;

    let mut handles = Vec::with_capacity(100);

    let handle = tokio::spawn({
        let ipfs = ipfs.clone();
        let cid = cid.to_string();

        async move { ipfs.pin_add(&cid, false).await }
    });
    handles.push(handle);

    if let Some(content_feed) = content_feed {
        if let Ok(res) = ipfs
            .name_resolve(Some(&content_feed.to_string()), false, false)
            .await
        {
            println!("Getting Content Feed...");

            let handle = tokio::spawn({
                let ipfs = ipfs.clone();
                let cid = res.path.clone();

                async move { ipfs.pin_add(&cid, false).await }
            });
            handles.push(handle);

            if let Ok(feed) = ipfs_dag_get_node_async::<FeedAnchor>(&ipfs, &res.path).await {
                for ipld in feed.content.into_iter() {
                    let ipfs = ipfs.clone();

                    let handle =
                        tokio::spawn(
                            async move { ipfs.pin_add(&ipld.link.to_string(), true).await },
                        );

                    handles.push(handle);
                }
            }
        } else {
            println!("Cannot Resolve Content Feed");
        }
    }

    if let Some(comments) = comments {
        println!("Resolving Comments...");

        if let Ok(res) = ipfs
            .name_resolve(Some(&comments.to_string()), false, false)
            .await
        {
            let handle = tokio::spawn({
                let ipfs = ipfs.clone();
                let cid = res.path.clone();

                async move { ipfs.pin_add(&cid, false).await }
            });
            handles.push(handle);

            println!("Getting Comments...");

            if let Ok(comments) = ipfs_dag_get_node_async::<Commentary>(&ipfs, &res.path).await {
                for ipld in comments.comments.into_values().flatten() {
                    let ipfs = ipfs.clone();

                    let handle =
                        tokio::spawn(
                            async move { ipfs.pin_add(&ipld.link.to_string(), false).await },
                        );

                    handles.push(handle);
                }
            }
        } else {
            println!("Cannot Resolve Comments");
        }
    }

    pin(&ipfs, Some(identity), &mut handles);
    pin(&ipfs, friends, &mut handles);
    pin(&ipfs, live, &mut handles);
    pin(&ipfs, bans, &mut handles);
    pin(&ipfs, mods, &mut handles);

    println!("Pinning...");

    for handle in handles {
        match handle.await {
            Ok(result) => match result {
                Ok(_) => continue,
                Err(ipfs_err) => {
                    eprintln!("❗ IPFS: {}", ipfs_err);
                    continue;
                }
            },
            Err(e) => {
                eprintln!("❗ Tokio: {}", e);
                continue;
            }
        }
    }

    println!("✅ Pinned Beacon {}", &cid);

    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct Unpin {
    /// Beacon CID.
    #[structopt(short, long)]
    cid: Cid,
}

async fn unpin_beacon(args: Unpin) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    let Unpin { cid } = args;

    println!("Getting Beacon...");

    let beacon = ipfs_dag_get_node_async(&ipfs, &cid.to_string()).await?;

    let Beacon {
        identity,
        content_feed,
        comments,
        friends,
        live,
        bans,
        mods,
    } = beacon;

    let mut handles = Vec::with_capacity(100);

    let handle = tokio::spawn({
        let ipfs = ipfs.clone();
        let cid = cid.to_string();

        async move { ipfs.pin_rm(&cid, false).await }
    });
    handles.push(handle);

    println!("Resolving Content Feed...");

    if let Some(content_feed) = content_feed {
        if let Ok(res) = ipfs
            .name_resolve(Some(&content_feed.to_string()), false, false)
            .await
        {
            let handle = tokio::spawn({
                let ipfs = ipfs.clone();
                let cid = res.path.clone();

                async move { ipfs.pin_rm(&cid, false).await }
            });
            handles.push(handle);

            println!("Getting Content Feed...");

            if let Ok(feed) = ipfs_dag_get_node_async::<FeedAnchor>(&ipfs, &res.path).await {
                for ipld in feed.content.into_iter() {
                    let ipfs = ipfs.clone();

                    let handle =
                        tokio::spawn(
                            async move { ipfs.pin_rm(&ipld.link.to_string(), true).await },
                        );

                    handles.push(handle);
                }
            }
        } else {
            println!("Cannot Resolve Content Feed");
        }
    }

    if let Some(comments) = comments {
        println!("Resolving Comments...");

        if let Ok(res) = ipfs
            .name_resolve(Some(&comments.to_string()), false, false)
            .await
        {
            let handle = tokio::spawn({
                let ipfs = ipfs.clone();
                let cid = res.path.clone();

                async move { ipfs.pin_rm(&cid, false).await }
            });
            handles.push(handle);

            println!("Getting Comments...");

            if let Ok(comments) = ipfs_dag_get_node_async::<Commentary>(&ipfs, &res.path).await {
                for ipld in comments.comments.into_values().flatten() {
                    let ipfs = ipfs.clone();

                    let handle =
                        tokio::spawn(
                            async move { ipfs.pin_rm(&ipld.link.to_string(), false).await },
                        );

                    handles.push(handle);
                }
            }
        } else {
            println!("Cannot Resolve Comments");
        }
    }

    unpin(&ipfs, Some(identity), &mut handles);
    unpin(&ipfs, friends, &mut handles);
    unpin(&ipfs, live, &mut handles);
    unpin(&ipfs, bans, &mut handles);
    unpin(&ipfs, mods, &mut handles);

    println!("Unpinning...");

    for handle in handles {
        match handle.await {
            Ok(result) => match result {
                Ok(_) => continue,
                Err(ipfs_err) => {
                    eprintln!("❗ IPFS: {}", ipfs_err);
                    continue;
                }
            },
            Err(e) => {
                eprintln!("❗ Tokio: {}", e);
                continue;
            }
        }
    }

    println!("✅ Unpinned Beacon {}", &cid);

    Ok(())
}

fn pin(
    ipfs: &IpfsClient,
    ipns: Option<Cid>,
    handles: &mut Vec<JoinHandle<Result<PinAddResponse, Error>>>,
) {
    if let Some(ipns) = ipns {
        let handle = tokio::spawn({
            let ipfs = ipfs.clone();
            let ipns = ipns.to_string();

            async move {
                let res = ipfs.name_resolve(Some(&ipns), false, false).await?;

                ipfs.pin_add(&res.path, false).await
            }
        });

        handles.push(handle);
    }
}

fn unpin(
    ipfs: &IpfsClient,
    ipns: Option<Cid>,
    handles: &mut Vec<JoinHandle<Result<PinRmResponse, Error>>>,
) {
    if let Some(ipns) = ipns {
        let handle = tokio::spawn({
            let ipfs = ipfs.clone();
            let ipns = ipns.to_string();

            async move {
                let res = ipfs.name_resolve(Some(&ipns), false, false).await?;

                ipfs.pin_rm(&res.path, false).await
            }
        });

        handles.push(handle);
    }
}

async fn create_ipns_link<T>(
    ipfs: &IpfsClient,
    name: &str,
    key: &str,
    key_list: &ipfs_api::response::KeyPairList,
    data: Option<T>,
) -> Result<Cid, Error>
where
    T: Default + Serialize,
{
    let link = match search_keypairs(key, key_list) {
        Some(kp) => kp.id.to_owned(),
        None => {
            println!("Generating {} IPNS Key...", name);

            let ipns_link = generate_key(ipfs, key).await?;

            println!("Updating {} IPNS Link...", name);

            ipns_link
        }
    };

    let cid = Cid::try_from(link).expect("Cannot Serialize CID");

    if let Some(data) = data {
        update_ipns(ipfs, key, &data).await?;
    } else {
        update_ipns(ipfs, key, &T::default()).await?;
    }

    println!("✅ {} IPNS Link => {}", name, &cid);

    Ok(cid)
}

async fn generate_key(ipfs: &IpfsClient, key: &str) -> Result<String, Error> {
    let res = ipfs
        .key_gen(
            key,
            KeyType::Ed25519,
            64, /* Don't think this does anything... */
        )
        .await?;

    Ok(res.id)
}
