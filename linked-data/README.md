# Linked Data
Most of the code is annotated but here's the overview.

## Beacon
Mostly metadata and IPNS links. Lists of videos, past streams, banned users, moderators, etc...

## Streams
A video node contains links to segments of videos of all quality. As video is streamed, new video nodes are created and linked to previous ones. A special node contains the stream setup data; codecs, qualities, initialization segments, etc...

## Videos
Timecode nodes are created at specific intervals and linked together to form a structure around the video allowing it to be addressable by timecode. Video clips are subgraph of the whole. 

## Chat
Display Name and GossipSub Peer ID are signed using Ethereum Keys then the address, name, id, and signature are added to IPFS returning a CID. When receiving a message the CID is used to fetch and verify that IDs matches and signature is correct.

## Moderation
Moderator send ban message to users via GossipSub. The message is signed as if a chat message. The beacon is updated with the new lists.

## Blog
Micro-blogging & long form. Listed in content feed.

## Content Feed
Mutable content feed, create, updates or delete media.

## Comments
Comments are crypto-signed, link to the original content and optionally other comments. The list of comments is keep in sync with content feed.
