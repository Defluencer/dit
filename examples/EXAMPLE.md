# Experimental Use

## Setup
 - Make sure your broadcast software output is set to FFMPEG local address. Default=udp://127.0.0.1:2525
 - Make sure FFMPEG is on PATH. (Optional) To use a custom FFMPEG script remove the "ffmpeg" object from livelike_config.json
 - Compile streamer-app, in debug mode.
 - Bundle streamer-app & livelike_config.json together.
 - Set "streamer_peer_id" to the peer id of the ipfs node that will be used to stream.
 - Set "gossipsub_topic" to something unique to you.
 - Set "streamerPeerId" & "gossipsubTopic" in index.js to the same values.
 - Add the web-app folder to IPFS.
 - Use the IPFS link to share the stream.

 ## Streamer
 - Start IPFS with PUBSUB enabled.
 - Start streamer-app.
 - Start your broadcast software.
 - Start Streaming!

 ## Viewers
 - Start IPFS with PUBSUB enabled.
 - Set CORS.
 - Open IPFS link.