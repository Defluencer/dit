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
    /// CID of content being commented on.
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

    println!("Pinning...");

    let cc = comment_cid.to_string();
    if let Err(e) = ipfs.pin_add(&cc, false).await {
        eprintln!("❗ IPFS could not pin {}. Error: {}", cc, e);
    }

    println!("Updating Comment List...");

    let (old_comments_cid, mut list) = get_from_ipns::<Commentary>(&ipfs, COMMENTS_KEY).await?;

    match list.comments.get_mut(&origin) {
        Some(vec) => vec.push(comment_cid.into()),
        None => {
            list.comments.insert(origin, vec![comment_cid.into()]);
        }
    }

    update_ipns(&ipfs, COMMENTS_KEY, &list).await?;

    println!("Unpinning Old List...");

    let occ = old_comments_cid.to_string();
    if let Err(e) = ipfs.pin_rm(&occ, false).await {
        eprintln!("❗ IPFS could not unpin {}. Error: {}", occ, e);
    }

    println!("✅ Added Comment {}", comment_cid);

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

    let (old_comments_cid, mut list) = get_from_ipns::<Commentary>(&ipfs, COMMENTS_KEY).await?;

    let vec = match list.comments.get_mut(&origin) {
        Some(vec) => vec,
        None => return Err(Error::Uncategorized("Origin Not Found".into())),
    };

    let index = match vec.iter().position(|&ipld| ipld == comment.into()) {
        Some(idx) => idx,
        None => return Err(Error::Uncategorized("Index Not Found".into())),
    };

    vec.remove(index);

    if vec.is_empty() {
        list.comments.remove(&origin);
    }

    println!("Updating Comment List...");

    update_ipns(&ipfs, COMMENTS_KEY, &list).await?;

    println!("Unpinning Old List...");

    let occ = old_comments_cid.to_string();
    if let Err(e) = ipfs.pin_rm(&occ, false).await {
        eprintln!("❗ IPFS could not unpin {}. Error: {}", occ, e);
    }

    println!("✅ Removed Comment {}", comment);

    Ok(())
}
