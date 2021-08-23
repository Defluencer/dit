use crate::utils::dag_nodes::{get_from_ipns, ipfs_dag_put_node_async, update_ipns};

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;

use linked_data::comments::{Comment, Commentary};

use cid::Cid;

use structopt::StructOpt;

pub const COMMENTS_KEY: &str = "comments";

#[derive(Debug, StructOpt)]
pub struct Comments {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Add a new comment.
    Add(AddComment),

    /// Remove an old comment.
    Remove(RemoveComment),
}

pub async fn comments_cli(cli: Comments) {
    let res = match cli.cmd {
        Command::Add(add) => add_comment(add).await,
        Command::Remove(remove) => remove_comment(remove).await,
    };

    if let Err(e) = res {
        eprintln!("❗ IPFS: {:#?}", e);
    }
}

#[derive(Debug, StructOpt)]
pub struct AddComment {
    /// CID of content beign commented on.
    #[structopt(short, long)]
    origin: Cid,

    /// CID of comment being replied to.
    #[structopt(short, long)]
    reply: Option<Cid>,

    /// Content of your comment.
    #[structopt(short, long)]
    comment: String,
}

async fn add_comment(command: AddComment) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    let AddComment {
        origin,
        reply,
        comment,
    } = command;

    let reply = reply.map(|rep| rep.into());
    let comment = Comment::create(origin.into(), reply, comment);
    let comment_cid = ipfs_dag_put_node_async(&ipfs, &comment).await?;

    println!("Adding Comment {:?}", &comment_cid);

    println!("Pinning...");

    ipfs.pin_add(&comment_cid.to_string(), false).await?;

    println!("Updating Comment List...");

    let (old_comments_cid, mut comments) = get_from_ipns::<Commentary>(&ipfs, COMMENTS_KEY).await?;

    match comments.map.get_mut(&origin) {
        Some(vec) => vec.push(comment_cid.into()),
        None => {
            comments.map.insert(origin, vec![comment_cid.into()]);
        }
    }

    update_ipns(&ipfs, COMMENTS_KEY, &comments).await?;

    println!("Unpinning Old List...");

    ipfs.pin_rm(&old_comments_cid.to_string(), false).await?;

    println!("✅ Comment Added");

    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct RemoveComment {
    /// CID of the content commented on.
    #[structopt(short, long)]
    origin: Cid,

    /// CID of comment to remove.
    #[structopt(short, long)]
    comment: Cid,
}

async fn remove_comment(command: RemoveComment) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    let RemoveComment { origin, comment } = command;

    println!("Removing Comment {:?}", &comment);

    let (old_comments_cid, mut comments) = get_from_ipns::<Commentary>(&ipfs, COMMENTS_KEY).await?;

    let vec = match comments.map.get_mut(&origin) {
        Some(vec) => vec,
        None => return Err(Error::Uncategorized("Origin Not Found".into())),
    };

    let index = match vec.iter().position(|&ipld| ipld == comment.into()) {
        Some(idx) => idx,
        None => return Err(Error::Uncategorized("Index Not Found".into())),
    };

    vec.remove(index);

    println!("Updating Comment List...");

    update_ipns(&ipfs, COMMENTS_KEY, &comments).await?;

    println!("Unpinning Old List...");

    ipfs.pin_rm(&old_comments_cid.to_string(), false).await?;

    println!("✅ Comment Removed");

    Ok(())
}
