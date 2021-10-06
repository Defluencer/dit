# Linked Data

## Beacon

Directory of user content and metadata.
Since this object should not change it can be used as a unique identifier.
Always resolve the IPNS address to get the most up to date content.
If using IPNS over PubSub, you can also subscribe to a IPNS topic for live update.
Use friend list to crawl the social web.

### IPLD Schemas
```
type Beacon struct {
    identity IPNS
    content_feed optional IPNS
    comments optional IPNS
    live optional IPNS
    friends optional IPNS
    bans optional IPNS
    mods optional IPNS
}

type IPNS string # IPNS address
```
## Identity

A user name and avatar.

### IPLD Schemas
```
type Identity struct {
    display_name String # Your choosen name
    avatar Link # Link to an image
}
```

## Content Feed

An anchor for a user's content.
Chronological order is used.
Other indexing methods could be used.

### IPLD Schemas
```
type FeedAnchor struct {
    content [Media]
}

type Media union {
    | &MicroPost link
    | &FullPost link
    | &VideoMetadata link
} representation kinded
```
## Comments

An anchor for a user's comments.
Indexed by the content they commented on.
Other indexing methods could be used.

### IPLD Schemas
```
type Commentary struct {
    "comments": {String:[&Comment]} # Keys are CIDs of the content being commented on.
}

type Comment struct {
    timestamp Int # Unix Time
    author &Beacon
    origin Link # CID of content being commented on.
    comment String
}
```
## Friends

A list of friends you follow.

### IPLD Schemas
```
type Friendlies struct {
  friends [Friend] 
}

type Friend union {
    | ENS string # ENS domain name linked to Beacon
    | Beacon link # Link to Beacon
} representation kinded

type ENS string
type Beacon link
```
## Chat Moderation

Moderator can send ban/mod messages via PubSub.
The message is crypto-signed.

### IPLD Schemas
```
type Bans struct {
    banned [ETHAddress]
}

type Moderators struct {
    "mods": [ETHAddress]
}

type ETHAddress bytes # Ethereum address are 20 bytes.
```
## Chat
Display Name and GossipSub Peer ID are signed using Ethereum Keys then the address, name, id, and signature are added to IPFS returning a CID.
When receiving a message the CID is used to fetch and verify that IDs matches and signature is correct.

### IPLD Schemas
//TODO

## Streams
A video node contains links to segments of videos of all quality. As video is streamed, new video nodes are created and linked to previous ones.
A special node contains the stream setup data; codecs, qualities, initialization segments, etc...

### IPLD Schemas
```
type VideoNode struct {
    tracks {String:Link} # Name of the track egg "audio" or "1080p60" & link to video segment data
    setup optional &SetupNode
    previous optional &VideoNode
}

type SetupNode struct {
    tracks [Track] # Sorted from lowest to highest bitrate.
}

type Track struct {
    name String
    codec String
    init_seg Link
    bandwidth Int
}
```

## Videos
Timecode nodes are created at specific intervals and linked together to form a structure around the video allowing it to be addressable by timecode.
Video clips are subgraph of the whole.

### IPLD Schemas
```
type VideoMetadata struct {
    timestamp Int # Unix time
    duration Float
    image Link # Poster & Thumbnail
    video &TimeCodeNode
    author &Beacon
    title String
}

type TimeCodeNode struct {
    time &DayNode
}

type DayNode struct {
    hour [&HourNode]
}

type HourNode struct {
    minute [&MinuteNode]
}

type MinuteNode struct {
    second [&SecondNode]
}

type SecondNode struct {
    video &VideoNode
    chat [&ChatMessage]
}
```
## Blog
Micro-blogging & long form via markdown files.

### IPLD Schemas
```
type MicroPost struct {
    timestamp Int # Unix Time
    author &Beacon
    content String
}

type FullPost struct {
    timestamp Int # Unix Time
    author &Beacon
    content Link # Link to markdown file
    image Link
    title String
}
```