# Live-Like
The Live-Like project aim to decentralize live streaming. Provide the same popular moderation tools and allow content creators to receive 100% of users donations.

## Roadmap
- Live Streaming: Mostly working but could use some polish. Some customizations.
- VOD: Can save stream as timecode adressable video. Can't view yet.
- Live Chat: In Design.
- Moderation: In Design.
- Community: Some thoughts.
- Ecosystem: Wild dreams!

## Setup
 - Make sure your broadcast software output is set to FFMPEG local address.
 - Make sure FFMPEG is on PATH. (Optional) To use a custom FFMPEG script remove the "ffmpeg" object from livelike_config.json
 - Compile streamer-app, in debug mode.
 - Bundle streamer-app & livelike_config.json together.
 - Set "streamer_peer_id" to the peer id of the ipfs node that will be used to stream.
 - Set "gossipsub_topic" to something unique to you.
 - Set "streamerPeerId" & "gossipsubTopic" in index.js to the same values.
 - Add the web-app folder to IPFS or deploy as website.

## Streaming
 - Start IPFS with PUBSUB enabled.
 - Start streamer-app.
 - Start your broadcast software.
 - Stream!

## Viewing
 - Start IPFS with PUBSUB enabled.
 - Set IPFS CORS to allow calls from link or website.
 - Open IPFS link or website.

## V.O.D.
 - Stop your broadcast software.
 - Press Ctrl-c in streamer-app to save the stream locally.
 - Upload to Filecoin using final stream CID.

## License
Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## Financial Support
- https://gitcoin.co/grants/1084/live-like