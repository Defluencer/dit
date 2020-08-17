# Live-Like

## Decentralised Video Live Streaming

### Setup
 - Make sure your broadcast software output is set to FFMPEG local address. Default=udp://127.0.0.1:2525
 - Make sure FFMPEG is on PATH. (Optional) To use a custom FFMPEG script remove the "ffmpeg" object from livelike_config.json
 - Compile streamer-app, in debug mode.
 - Bundle streamer-app & livelike_config.json together.
 - Set "streamer_peer_id" to the peer id of the ipfs node that will be used to stream.
 - Set "gossipsub_topic" to something unique to you.
 - Set "streamerPeerId" & "gossipsubTopic" in index.js to the same values.
 - Add the web-app folder to IPFS.
 - Use the IPFS link to share the stream.

### Streamer
 - Start IPFS with PUBSUB enabled.
 - Start streamer-app.
 - Start your broadcast software.
 - Start Streaming!

### Viewers
 - Start IPFS with PUBSUB enabled.
 - Set CORS.
 - Open IPFS link.

### Long Term Goal
Any user with a IPFS node running and Companion browser extension can click the IPNS link and be served a WASM module containing a pre-configured IPFS node.

The node is configured for the streamer private network, to bootstrap their node with the streamer address and to subscribe to 2 topics; chat and video. The video topic is for receiving the latest video block hash, messages are signed with peerid proving identity. The chat topic is self-explanatory.

The streamer app publish the hash of an object containing the latest video blocks and a link to previous ones, it also receives chat messages. Transcoding is done at the source and the video blocks have different quality.

### License
Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.