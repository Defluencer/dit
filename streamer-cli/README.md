# Content
IPFS daemon must be running first. Command: ```ipfs daemon --enable-pubsub-experiment --enable-namesys-pubsub```

## Beacon
A beacon make your content discoverable and updateable.
- Command: ```streamer-cli beacon --help``` for more info.

## Moderation
Ban & moderator lists can be managed using commands.
- Command: ```streamer-cli moderation --help ``` for more info.

## Content Feed
Add, update or delete content from your feed.
- Command: ```streamer-cli content-feed --help``` for more info

## Availability
The beacon and all your content must be reachable at all times. To achieve this you should leave your IPFS daemon running 24/7 and others can also help you by pinning some or all your content, the more the better. Because of the decentralized nature of IPFS, it does not matter who has your data or how much of it, it cannot be modified and everyone will help redistribute it.

## Ethereum Name Service
If you already have a domain, the beacon CID can be used with ENS to associate your name to your content. Link the beacon CID to a subdomain called "defluencer".

# How To

## Video Live Streaming
- Start IPFS with PubSub enabled. Command: ```ipfs daemon --enable-pubsub-experiment```
- Start in live streaming mode. Command: ```streamer-cli stream```
- Run ```ffmpeg_live.sh``` or custom ffmpeg script.
- With your broadcast software output set to ffmpeg. Default: ```rtmp://localhost:2525```
- Start Streaming!
- When done streaming stop your broadcast software.
- Press Ctrl-c in streamer-cli window to save.
- Use the CLI to create metadata. Command: ```streamer-cli content-feed --help``` for more info.

## Pre-recorded Video
- Start IPFS. Command: ```ipfs daemon```
- Start in file mode. Command: ```streamer-cli file```
- Run ```ffmpeg_file.sh``` or custom ffmpeg script.
- Wait until the video is processed.
- Press Ctrl-c in streamer-cli window to save.
- Use the CLI to create metadata. Command: ```streamer-cli content-feed --help``` for more info.

# Technical

## Requirements
- [IPFS](https://docs.ipfs.io/install/command-line/#package-managers)
- [FFMPEG](https://ffmpeg.org/)
- Broadcasting software

## Configuration
config.json will be created automatically when creating beacon.
- Input socket address is the IP and Port the app will listen for FFMPEG on.
- Topics are used for live stream and chat.

## FFMPEG
- Output must be HLS.
- Must use fragmented mp4. (fmp4)
- Media segments length must be 1 second.
- Each track and folder must be named like so. "TRACK_NAME/SEGMENT_INDEX.m4s". egg ```1080p60/24.m4s```
- Audio track must standalone and be named "audio".
- Must produce a master playlist containing all tracks.

Due to a bug in FFMPEG, original videos cannot be in .mkv containers, missing metadata will cause missing tracks in HLS master playlist.

Refer to my scripts for inspiration in creating your own.