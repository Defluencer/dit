# Content
IPFS daemon must be running first. Command: ```ipfs daemon --enable-pubsub-experiment --enable-namesys-pubsub```

## Beacon
A content beacon make your content discoverable.
- Command: ```streamer-app beacon --help``` for more info.

## Videos
Video metadata can be created, updated and deleted using commands.
- Command: ```streamer-app video --help``` for more info.

## Availability
The content beacon and all your content must be reachable at all times. To achieve this you should leave your IPFS daemon running 24/7 and others can also help you by pinning some or all your content, the more the better. Because of the decentralized nature of IPFS, it does not matter who has your data or how much of it, it cannot be modified and everyone will help redistribute it.

## Ethereum Name Service
If you already have a domain, the beacon CID can be used with ENS to associate your name to your content. For use with defluencer.eth you must create a subdomain called defluencer and put the beacon CID in your records. It will make your name searchable on the website.

# How To

## Video Live Streaming
- Start IPFS with PubSub enabled. Command: ```ipfs daemon --enable-pubsub-experiment```
- Start DIT in live streaming mode. Command: ```streamer-app stream```
- Run example or custom ffmpeg script.
- With your broadcast software output set to ffmpeg. Default: ```rtmp://localhost:2525```
- Start Streaming!
- When done streaming stop your broadcast software.
- Wait 30 seconds to catch up with chat messages.
- Press Ctrl-c in streamer-app window to save.

## Pre-recorded Video
- Start IPFS. ```Command: ipfs daemon```
- Start DIT in file mode. Command: ```streamer-app file```
- Run example or custom ffmpeg script.
- Wait until the video is processed.
- Press Ctrl-c in streamer-app window to save.

# Technical

## Requirements
- [IPFS](https://docs.ipfs.io/install/command-line/#package-managers)
- [FFMPEG](https://ffmpeg.org/)
- Broadcasting software

## Configuration
config.json will be created automatically or can be created manually.
```
{
  "input_socket_addr": "127.0.0.1:2526",
  "archive": {},
  "video": {
    "pubsub_topic": "defluencer_live_video"
  },
  "chat": {
    "pubsub_topic": "defluencer_live_chat"
  }
}
```
- Input socket address is the IP and Port the app will listen for FFMPEG on.
- Topics are used for live stream and chat. Choose some unique names.

## FFMPEG
- Output must be HLS.
- Media segments length must be 1 second.
- Each track must be named and structured like this "TRACK_NAME/SEGMENT_INDEX". egg ```1080p60/24.m4s```
- Must use fragmented mp4. (fmp4)
- Must produce a master playlist containing all tracks.
- Audio track must standalone and be named "audio".