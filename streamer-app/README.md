## Requirements
- [Rust + Cargo](https://www.rust-lang.org/tools/install)
- [IPFS](https://docs.ipfs.io/install/command-line/#package-managers)
- [FFMPEG](https://ffmpeg.org/)
- Broadcasting software

## Streaming Setup
- Compile streamer-app.
- Make sure FFMPEG is installed and on PATH.
- Broadcast software set to FFMPEG using rtmp. (default rtmp://localhost:2525)

## Configuration
- config.json will be created automatically on first run but can be created manually.
```
{
  "gossipsub_topics": {
    "live_video": "defluencer_live_video",
    "live_chat": "defluencer_live_chat"
  },
  "addresses": {
    "app_addr": "127.0.0.1:2526",
    "ffmpeg_addr": "127.0.0.1:2525"
  },
  "segment_duration": 4
}
```
- Topics are used for live stream and chat. Choose some unique names.
- Addresses indicate the IP and Port apps will listen on.
    - Optionally FFMPEG address can be omited. It allow a custom script to be used.
- Segment duration is how long one media segment last in seconds. Make sure it's the same eveywhere.

## Live
- Start IPFS with PubSub enabled. Command: ipfs daemon --enable-pubsub-experiment
- Start streamer-app.
- Start your broadcast software.
- Stream!

## Saving Live Streams
- Stop your broadcast software.
- Wait 15 seconds for streamer-app to catch up with chat messages.
- Pressing Ctrl-c in streamer-app will save the stream locally.