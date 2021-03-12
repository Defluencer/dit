## Requirements
- [Rust + Cargo](https://www.rust-lang.org/tools/install)
- [IPFS](https://docs.ipfs.io/install/command-line/#package-managers)
- [FFMPEG](https://ffmpeg.org/)
- Broadcasting software

## Configuration
- config.json will be created automatically on first run but can be created manually.
```
{
  "input_socket_addrs": "127.0.0.1:2526",
  "archive": {
    "archive_live_chat": true,
    "segment_duration": 4
  },
  "video": {
    "pubsub_topic": "defluencer_live_video"
  },
  "chat": {
    "pubsub_topic": "defluencer_live_chat"
  },
  "ffmpeg": {
    "output_socket_addrs": "localhost:2526",
    "input_socket_addrs": "localhost:2525"
  }
}
```
- Topics are used for live stream and chat. Choose some unique names.
- Archive Configuration (can be omited to disable archiving)
  - Archive live chat set to false will disable live chat archiving.
  - Segment duration is how long one media segment last in seconds. Make sure it's the same eveywhere.
- Video Configuration
  - Optionaly pubsub topic can be omited to disable live streaming.
- FFMPEG Configuration (Can be omited. It allow a custom ffmpeg script to be used.)
  - Optionaly input can be omited to ingess video files. Command: streamer-app Path/To/Video/File

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
- Wait until the video is processed.
- Press Ctrl-c in streamer-app to save the video.