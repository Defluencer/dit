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
  "archive": {
    "archive_live_chat": true,
    "segment_duration": 4
  },
  "video": {
    "live_stream": true
  }
}
```
- Topics are used for live stream and chat. Choose some unique names.
- Addresses indicate the IP and Port apps will listen on.
  - Optionally FFMPEG address (ffmpeg_addr) can be omited. It allow a custom script to be used.
- Archive Configuration (can be omited to disable archiving)
  - Archive live chat set to false will disable live chat archiving.
  - Segment duration is how long one media segment last in seconds. Make sure it's the same eveywhere.
- Video Configuration
  - Optionaly pubsub topic can be omited to disable live streaming.
- Chat Configuration

## Video Live Streaming
Enable archiving and live streaming in config.
- Start IPFS with PubSub enabled. Command: ipfs daemon --enable-pubsub-experiment
- Start streamer-app.
- Start your broadcast software.
- Stream!
- Stop your broadcast software.
- Wait 30 seconds to catch up with chat messages.
- Press Ctrl-c in streamer-app to save the stream locally.

## Pre-recorded Video
Disable; live streaming, chat and ffmpeg in config.
- Start IPFS. Command: ipfs daemon
- Start streamer-app.
- Start FFMPEG using custom script.
- Wait until the video is processed.
- Press Ctrl-c in streamer-app to save the video.