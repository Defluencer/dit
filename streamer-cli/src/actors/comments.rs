use crate::cli::content::{COMMENTS_KEY, FEED_KEY};
use crate::utils::dag_nodes::{
    get_from_ipns, ipfs_dag_get_node_async, ipfs_dag_put_node_async, update_ipns,
};

use tokio_stream::StreamExt;

use ipfs_api::response::Error;
use ipfs_api::response::PubsubSubResponse;
use ipfs_api::IpfsClient;

use linked_data::comments::{Comment, Comments, CommentsAnchor};
use linked_data::feed::FeedAnchor;
use linked_data::signature::SignedMessage;

pub struct CommentsAggregator {
    ipfs: IpfsClient,
}

impl CommentsAggregator {
    pub fn new(ipfs: IpfsClient) -> Self {
        Self { ipfs }
    }

    pub async fn start(&mut self) {
        let mut stream = self.ipfs.pubsub_sub(COMMENTS_KEY, true);

        println!("✅ Comments System Online");

        while let Some(result) = stream.next().await {
            //TODO find way to cleanly close the stream.

            match result {
                Ok(response) => self.on_pubsub_message(response).await,
                Err(error) => {
                    eprintln!("{}", error);
                    continue;
                }
            }
        }

        println!("❌ Comments System Offline");
    }

    async fn on_pubsub_message(&self, msg: PubsubSubResponse) {
        let PubsubSubResponse {
            from,
            data,
            seqno: _,
            topic_ids: _,
            unrecognized: _,
        } = msg;

        let (_peer, data) = match (from, data) {
            (Some(from), Some(data)) => (from, data),
            (Some(_), None) => return,
            (None, Some(_)) => return,
            (None, None) => return,
        };

        let signed_comment: SignedMessage<Comment> = match serde_json::from_slice(&data) {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("❗ Comment: deserialization failed {}", e);
                return;
            }
        };

        if !signed_comment.verify() {
            eprintln!("❗ Comment: wrong signature");
            return;
        }

        //TODO filter comment

        if let Err(e) = self.add_comment(signed_comment).await {
            eprintln!("❗ IPFS: {:#?}", e);
        }
    }

    /// Get the index of the content being commented on,
    /// add the new comment to the list then
    /// update the content comments anchor.
    async fn add_comment(&self, signed_comment: SignedMessage<Comment>) -> Result<(), Error> {
        let ((_, feed), (old_comments_anchor, mut comments_anchor)) = tokio::try_join!(
            get_from_ipns::<FeedAnchor>(&self.ipfs, FEED_KEY),
            get_from_ipns::<CommentsAnchor>(&self.ipfs, COMMENTS_KEY)
        )?;

        let index = match feed
            .content
            .iter()
            .rposition(|&link| link == signed_comment.data.origin)
        {
            Some(idx) => idx,
            None => return Err(Error::Uncategorized("Content Not Found".into())),
        };

        let old_comments = comments_anchor.links[index].link.to_string();
        let mut comments = ipfs_dag_get_node_async::<Comments>(&self.ipfs, &old_comments).await?;

        let cid = ipfs_dag_put_node_async(&self.ipfs, &signed_comment).await?;
        comments.list.push(cid.into());

        let cid = ipfs_dag_put_node_async(&self.ipfs, &comments).await?;
        comments_anchor.links[index] = cid.into();

        let comments_cid_string = cid.to_string();

        tokio::try_join!(
            update_ipns(&self.ipfs, COMMENTS_KEY, &comments_anchor),
            self.ipfs.pin_add(&comments_cid_string, true)
        )?;

        let old_comments_anchor = old_comments_anchor.to_string();

        tokio::try_join!(
            self.ipfs.pin_rm(&old_comments_anchor, false),
            self.ipfs.pin_rm(&old_comments, true),
        )?;

        Ok(())
    }
}
