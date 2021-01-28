## Requirements
- [Rust + Cargo](https://www.rust-lang.org/tools/install)
- [IPFS](https://docs.ipfs.io/install/command-line/#package-managers)
- [FFMPEG](https://ffmpeg.org/)
- Broadcasting software

## Streaming Setup
- Compile streamer-app in debug mode.
- Broadcast software set to FFMPEG using rtmp. (Default -> rtmp://127.0.0.1:2525)
- Make sure FFMPEG is installed and on PATH.

## Live
- Start IPFS with PubSub enabled.
- Start streamer-app. (It will in turn start FFMPEG)
- Start your broadcast software.
- Stream!

## V.O.D.
- Stop your broadcast software.
- Wait 15 seconds for streamer-app to catch up.
- Press Ctrl-c in streamer-app to save the stream locally (video & chat).