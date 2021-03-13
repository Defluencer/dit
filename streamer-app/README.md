## How it works
FFMPEG is used to transcode the video or stream from the broadcasting software then the application add this data to IPFS and publish it using gossipsub.

## Requirements
- [Rust + Cargo](https://www.rust-lang.org/tools/install)
- [IPFS](https://docs.ipfs.io/install/command-line/#package-managers)
- [FFMPEG](https://ffmpeg.org/)
- Broadcasting software

## Command Line Interface
Use --help for more info.

## Configuration
- config.json will be created automatically on first run but can be created manually.
```
{
  "input_socket_addr": "127.0.0.1:2526",
  "archive": {
    "segment_duration": 4
  },
  "video": {
    "pubsub_topic": "defluencer_live_video"
  },
  "chat": {
    "pubsub_topic": "defluencer_live_chat"
  }
}
```
- Input socket address is the IP and Port the app will listen for FFMPEG on.
- Segment duration is how long one media segment last in seconds. Also called Keyframe Interval.
- Topics are used for live stream and chat. Choose some unique names.

## Video Live Streaming
Set the output of your broadcasting software to the input of FFMPEG. Default rtmp://localhost:2525
- Start IPFS with PubSub enabled. Command: ipfs daemon --enable-pubsub-experiment
- Run script live_stream or custom script.
- Run script ffmpeg_hls_stream or custom script.
- Start your broadcast software.
- Stream!
- Stop your broadcast software.
- Wait 30 seconds to catch up with chat messages.
- Press Ctrl-c in streamer-app to save the stream locally.

## Pre-recorded Video
- Start IPFS. Command: ipfs daemon
- Run script file_stream or custom script.
- Run script ffmpeg_file_stream or custom script.
- Input path to video file when asked.
- Wait until the video is processed.
- Press Ctrl-c in streamer-app to save the video.