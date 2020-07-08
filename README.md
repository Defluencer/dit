# Live-Like
## Decentralised Video Live Streaming
### Overview
Any user with a IPFS node running and Companion browser extension can click the IPNS link and be served a WASM module containing a pre-configured IPFS node.

The node is configured for the streamer private network, to bootstrap their node with the streamer address and to subscribe to 2 topics; chat and video. The video topic is for receiving the latest video block hash, messages are signed with peerid proving identity. The chat topic is self-explanatory.

The streamer app publish the hash of an object containing the latest video blocks and a link to previous ones, it also receives chat messages. Transcoding is done at the source and the video blocks have different quality.

Currently WIP...