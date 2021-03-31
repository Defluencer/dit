# Linked Data
Most of the code is annotated but here's the overview.

## Beacon
Metadata and lists of links to content. By broadcasting this, other peers can access your latest content.

## Stream
As video play, new nodes are created and linked to previous ones. A node contain data required to play a segment of video. A special node contains the stream setup data; codecs, qualities, initialization segments, etc...

## Video
Nodes are created at specific intervals and linked together to form a structure around the video allowing it to be addressable by timecode.