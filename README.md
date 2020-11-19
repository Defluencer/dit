# Live-Like
The Live-Like project aim to decentralize live streaming. Provide the same popular moderation tools and allow content creators to receive 100% of users donations.

## Roadmap
- Live Streaming: Mostly working.
- VOD: Can save stream as timecode adressable video. Can't view yet.
- Live Chat: Can send and receive messages. No identity yet.
- Moderation: In Design.
- Community: Some thoughts.
- Ecosystem: Wild dreams!

## Setup
### Streaming
- Customize template config.json then IPFS add, pin and name publish the config file.
- Broadcast software set to FFMPEG using rtmp. (egg: rtmp://127.0.0.1:2525)
- Make sure FFMPEG is on PATH.
- (Optional) Remove the "ffmpeg" object from config.json if using custom FFMPEG scripts.
### Web
- Customize Web-app.
- IPFS add and pin the www folder.
- Upload CID to ENS or other DNS.

## Live
- Start IPFS with PUBSUB enabled.
- Start streamer-app.
- Start your broadcast software.
- Stream!

## V.O.D.
- Stop your broadcast software.
- Wait 15 seconds for the app to catch up.
- Press Ctrl-c in streamer-app to save the stream locally (video & chat).
- Upload to Filecoin using final stream CID.

## Viewers
- Start IPFS with PUBSUB enabled.
- Set IPFS CORS to allow website.
- Open website.

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
- sionois.eth