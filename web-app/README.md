## Requirements
- [Rust + Cargo](https://www.rust-lang.org/tools/install)
- [IPFS](https://docs.ipfs.io/install/command-line/#package-managers)
- [Yew](https://yew.rs/docs/en/next/getting-started/project-setup)
- [Trunk](https://yew.rs/docs/en/next/getting-started/project-setup/using-trunk)

## Web-App
- Customize as needed.
- Change ENS name in the main.rs file.
- Compile with this command: trunk build --release
- Add and Pin the www folder to IPFS. Command: ```ipfs add --recursive --cid-version=1 www```
- Upload CID to Pinata Cloud, Temporal and/or host it yourself.
- Upload CID to ENS or other DNS.

## Testing
### Setup 1
IPFS natively in Brave. (live streams don't work, cannot enable pubsub)
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
      "http://<INSERT_CID_HERE>.ipfs.localhost:48084"
    ]
  }
},
```
- Restart browser

### Setup 2
IPFS + any browser
- [Install IPFS CLI](https://dist.ipfs.io/#go-ipfs)
- [Initialize IPFS](https://docs.ipfs.io/how-to/command-line-quick-start/#initialize-the-repository) with this command: ipfs init
- Allow CORS with these commands;
    - ipfs config --json API.HTTPHeaders.Access-Control-Allow-Methods '["GET", "POST", "PUT"]'
    - ipfs config --json API.HTTPHeaders.Access-Control-Allow-Origin '["http://localhost:5001", "http://127.0.0.1:5001", "http://<INSERT_CID_HERE>.ipfs.localhost:8080"]'
- [Start IPFS Daemon](https://docs.ipfs.io/reference/cli/#ipfs-daemon) with PubSub enabled using this command: ipfs daemon --enable-pubsub-experiment

### Dev Build
Checkout defluencer.eth on Ropsten testnet for a live example.