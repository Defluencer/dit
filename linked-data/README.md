# Linked Data
Most of the code is annotated but here's the overview.

Please refer to [this](https://ipld.io/docs/intro/hello-world/#diagram) for terminology.

## Beacon
Metadata and IPNS links.
Should not change.
Can be linked to a domain name.
Can be use as an unique identifier.

Beacon
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
- Display Name: User facing name.
- Topics: refers to the name of pubsub topics used for live streaming & chat.
    - Video: Keccak256 hash of display name concatened with _video
    - Chat: Keccak256 hash of display name concatened with _chat
- Peer ID: peer id of node used to live stream video as a Base58Btc string.
- Content Feed: IPNS link to content.
- Comments (Optional): IPNS link to comments.
- Friends List (Optional): IPNS link to friends.
- Chat Ban List (Optional): IPNS link to banned.
- Chat Moderator List (Optional): IPNS link to mods.

## Content Feed
Mutable content feed; create, updates or delete media.
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
- Content: list of IPLD links to some media content.

## Comments
Map of all comments.
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
- Comments: Map,
    - Keys: CIDs of the content being commented on.
    - Values: list of IPLD link to comments.

## Friends
List of ENS domain name or Beacon CIDs. Used to fetch your friends content and comments.

Friend List
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
Moderator send ban message to users via GossipSub. The message is signed as if a chat message. The beacon is updated with the new lists.

Banned List
```
{
    "banned": [
        ETH_ADDR,
        ETH_ADDR,
    ]
}
```

Moderator List
```
{
    "mods": [
        ETH_ADDR,
        ETH_ADDR,
    ]
}
```
- ETH_ADDR: ethereum address as 20 bytes.

## Chat
Display Name and GossipSub Peer ID are signed using Ethereum Keys then the address, name, id, and signature are added to IPFS returning a CID. When receiving a message the CID is used to fetch and verify that IDs matches and signature is correct.

Message
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
- Msg Type: could also be other stuff like a message to ban some user by a mod.

Chat ID
```
{
    "name": "NAME_HERE",
    "peer": "PEER_ID_HERE,
}
```

## Streams
A video node contains links to segments of videos of all quality. As video is streamed, new video nodes are created and linked to previous ones. A special node contains the stream setup data; codecs, qualities, initialization segments, etc...

Video Node
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

Setup Node
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
Timecode nodes are created at specific intervals and linked together to form a structure around the video allowing it to be addressable by timecode. Video clips are subgraph of the whole.

VideoMetadata
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

Timecode Node
```
{
    "time": {
        "/": "DAY_NODE_CID"
    }
}
```

Day Node
```
{
    "hour": {
        "/": "HOUR_NODE_CID"
    }
}
```

Hour Node
```
{
    "minute": {
        "/": "MINUTE_NODE_CID"
    }
}
```

Minute Node
```
{
    "second": {
        "/": "SECOND_NODE_CID"
    }
}
```

Second Node
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

MicroPost
```
{
    "timestamp": UNIX_TIME,
    "content": "MESSAGE_HERE"
}
```

FullPost
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