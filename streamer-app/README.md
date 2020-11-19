## Streaming Setup
- Compile streamer-app.
- Customize template config.json then IPFS add, pin and name publish the config file.
- Broadcast software set to FFMPEG using rtmp. (default: rtmp://127.0.0.1:2525)
- Make sure FFMPEG is on PATH.
- (Optional) Remove the "ffmpeg" object from config.json if using custom FFMPEG scripts.

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