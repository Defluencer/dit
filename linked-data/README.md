# Linked Data
Please refer to [this](https://ipld.io/docs/intro/hello-world/#diagram) for terminology.

All schemas are represented per block with all their nodes.

## Beacon 
Directory of user content and metadata.
Since this object should not change it can be used as a unique identifier.
Resolve IPNS links and/or subscribe to get updates.
Use friends list to crawl the network for content.
Always resolve the IPNS links to get the most up to date content.
If using IPNS over PubSUb, you can also subscribe to a IPNS link's topic for live update.

### IPLD Schemas
- Beacon
    - Display Name: name choosen at creation time.
    - Topics: refers to the name of pubsub topics used for live streaming & chat.
        - Video: Keccak256 hash of display name concatened with _video
        - Chat: Keccak256 hash of display name concatened with _chat
    - Peer ID: peer id of node used to live stream video as a Base58Btc string.
    - Content Feed: IPNS link to content feed.
    - Comments (Optional): IPNS link to comments.
    - Friends List (Optional): IPNS link to friends list.
    - Chat Ban List (Optional): IPNS link to banned users list.
    - Chat Moderator List (Optional): IPNS link to moderators list.
```
{
    "display_name": "",
    "topics": {
        "video": "",
        "chat": "",
    }
    "peer_id": "",
    "content_feed": "",
    "friends": "",
    "bans": "",
    "mods": "",
}
```
## Content Feed
A list of links to a user's content in chronological order.
### IPLD Schemas
- Feed Anchor
    - Content: list of IPLD links to some media content.
```
{
    "content": [
        {
            "/": "CONTENT_CID"
        },
        {
            "/": "CONTENT_CID"
        }
    ]
}
```
## Comments
A map of links to a user's comments keyed by content.

### IPLD Schemas
- Comments: Map,
    - Keys: CIDs of the content being commented on.
    - Values: list of IPLD link to comments.
```
{
    "comments": [
        {
            "CONTENT_CID": [
                {
                    "/": "COMMENT_CID",
                },
                {
                    "/": "COMMENT_CID",
                }
            ]
        },
        {
            "CONTENT_CID": [
                {
                    "/": "COMMENT_CID",
                },
                {
                    "/": "COMMENT_CID",
                }
            ]
        }
    ]
}
```
- Comment
    - Timestamp: number of second since 01/01/1970 00:00 UTC.
    - Origin: CID of content being commented on.
    - Reply: Optional CID of the comment being replied to.
    - Comment: text message.
```
{
    "timestamp": UNIX_TIME,
    "origin": {
        "/"; "CONTENT_CID"
    },
    "reply": {
        "/": "COMMENT_CID"
    },
    "comment": "blabla"
}
```
## Friends
A list of friends you follow.

### IPLD Schemas
- Friend list
    - Friend: Either a beacon CID OR a ENS domain name linked to a beacon CID.
```
{
  "friends": [
    {
      "friend": {
        "/": "BEACON_CID"
      }
    },
    {
      "friend": "my.domain.eth"
    }
  ]
}
```
## Chat Moderation
Moderator can send ban/mod messages via PubSub.
The message is crypto-signed.

### IPLD Schemas
- Banned List
    - ETH_ADDR: ethereum address as 20 bytes.
```
{
    "banned": [
        ETH_ADDR,
        ETH_ADDR,
    ]
}
```
- Moderator List
    - ETH_ADDR: ethereum address as 20 bytes.
```
{
    "mods": [
        ETH_ADDR,
        ETH_ADDR,
    ]
}
```
## Chat
Display Name and GossipSub Peer ID are signed using Ethereum Keys then the address, name, id, and signature are added to IPFS returning a CID.
When receiving a message the CID is used to fetch and verify that IDs matches and signature is correct.

### IPLD Schemas
- Message
    - Msg Type: message to ban, mod a user or jost text.
    - Origin: CID of crypto-signed Chat ID.
```
{
    "msg_type": {
        "message": "MESSAGE_HERE"
    },
    "origin": {
        "/": "SIGNED_CHAT_ID_CID"
    }
}
```
- Chat ID
```
{
    "name": "NAME_HERE",
    "peer": "PEER_ID_HERE,
}
```
## Streams
A video node contains links to segments of videos of all quality. As video is streamed, new video nodes are created and linked to previous ones.
A special node contains the stream setup data; codecs, qualities, initialization segments, etc...

### IPLD Schemas
- Video Node
    - Tracks: Map
        - Keys: name of the track egg "audio" or "1080p60"
        - Values: CID of the video segment.
    - Setup: CID of Setup Node
    - Previous: CID of the previous Video Node
```
{
    "tracks": [
        {
            "TRACK_NAME": {
                "/": "TRACK_CID"
            }
        },
        {
            "TRACK_NAME": {
                "/": "TRACK_CID"
            }
        }
    ],
    "setup": {
        "/": "SETUP_NODE_CID"
    },
    "previous": {
        "/": "PREVIOUS_VIDEO_NODE_CID"
    }
}
```
- Setup Node
   - Tracks: List
        - Name: name of the track. egg "audio" or "1080p60"
        - Codec: mime type and codec. egg "video/mp4; codecs="avc1.4d002a""
        - Initialization Segment: CID of the initialization segment.
        - Bandwidth: number of bit per second for this track.
```
{
    "tracks": [
        {
            "name": "TRACK_NAME",
            "codec": "CODEC_NAME",
            "initseg": {
                "/": "INIT_SEG_CID"
            },
            "bandwidth": BIT_PER_SECOND,
        },
        {
            "name": "TRACK_NAME",
            "codec": "CODEC_NAME",
            "initseg": {
                "/": "INIT_SEG_CID"
            },
            "bandwidth": BIT_PER_SECOND,
        }
    ]
}
```

## Videos
Timecode nodes are created at specific intervals and linked together to form a structure around the video allowing it to be addressable by timecode.
Video clips are subgraph of the whole.

### IPLD Schemas
- VideoMetadata
    - Timestamp: number of second since 01/01/1970 00:00 UTC.
    - Video: CID of Timecode node.
    - Image: CID of thumbnail image.
    - Duration: float number of second elapsed at th end of the video.
```
{
    "timestamp": UNIX_TIME,
    "video": {
        "/"; "TIMECODE_NODE_CID"
    },
    "image": {
        "/": "IMAGE_CID"
    },
    "title": "TITLE",
    "duration": DURATION_FRAC
}
```
- Timecode Node
```
{
    "time": {
        "/": "DAY_NODE_CID"
    }
}
```
- Day Node
```
{
    "hour": {
        "/": "HOUR_NODE_CID"
    }
}
```
- Hour Node
```
{
    "minute": {
        "/": "MINUTE_NODE_CID"
    }
}
```
- Minute Node
```
{
    "second": {
        "/": "SECOND_NODE_CID"
    }
}
```
- Second Node
```
{
    "video": {
        "/": "VIDEO_NODE_CID"
    },
    "chat": [
        {
            "/": "CHAT_MSG_CID"
        },
        {
            "/": "CHAT_MSG_CID"
        }
    ]
}
```
## Blog
Micro-blogging & long form via markdown files. 

### IPLD Schemas
- MicroPost
    - Timestamp: number of second since 01/01/1970 00:00 UTC.
    - Content: text message.
```
{
    "timestamp": UNIX_TIME,
    "content": "MESSAGE_HERE"
}
```
- FullPost
    - Timestamp: number of second since 01/01/1970 00:00 UTC.
    - Content: CID of markdown file.
    - Image: CID of banner and thumbnail image.
    - Title: title of the article.
```
{
    "timestamp": UNIX_TIME,
    "content": {
        "/": "MARKDOWN_FILE_CID"
    },
    "image": {
        "/": "IMAGE_CID"
    },
    "title": "TITLE"
}
```