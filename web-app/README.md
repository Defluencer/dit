## Requirements
- [Rust + Cargo](https://www.rust-lang.org/tools/install)
- [IPFS](https://docs.ipfs.io/install/command-line/#package-managers)
- [Yew](https://yew.rs/docs/en/next/getting-started/project-setup)
- [Trunk](https://yew.rs/docs/en/next/getting-started/project-setup/using-trunk)

## Web-App
- Customize as needed.
- Compile with this command: trunk build --release
- Add and Pin the www folder to IPFS using this command: ipfs add --recursive --cid-version=1 www
- Upload CID to ENS or other DNS.
- Upload CID to Pinata Cloud, Temporal and/or host it yourself.

## Testing
### Setup 1
IPFS natively in Brave. (live streams don't work)
- Install [Brave](https://brave.com/)
- Go to brave://settings
- Enable IPFS companion then when asked enable IPFS
- Click companion extension icon then click My Node
- Go to settings and replace
```
"API": {
  "HTTPHeaders": {}
},
```
with this
```
"API": {
  "HTTPHeaders": {
    "Access-Control-Allow-Methods": [
      "PUT",
      "POST",
      "GET"
    ],
    "Access-Control-Allow-Origin": [
      "http://localhost:45005",
      "http://127.0.0.1:45005",
      "https://webui.ipfs.io",
      "http://ADD_YOUR_CID_HERE.ipfs.localhost:48084"
    ]
  }
},
```
- Restart browser

### Setup 2
IPFS + any browser
- [Install IPFS CLI](https://docs.ipfs.io/install/command-line/#official-distributions)
- [Initialize IPFS](https://docs.ipfs.io/how-to/command-line-quick-start/#initialize-the-repository) with this command: ipfs init
- Allow CORS with these commands;
    - ipfs config --json API.HTTPHeaders.Access-Control-Allow-Methods '["GET", "POST", "PUT"]'
    - ipfs config --json API.HTTPHeaders.Access-Control-Allow-Origin '["http://localhost:5001", "http://127.0.0.1:5001", "http://ADD_YOUR_CID_HERE.ipfs.localhost:8080"]'
- [Start IPFS Daemon](https://docs.ipfs.io/reference/cli/#ipfs-daemon) with PubSub enabled using this command: ipfs daemon --enable-pubsub-experiment
- Input http://ADD_YOUR_CID_HERE.ipfs.localhost:8080 in your browser