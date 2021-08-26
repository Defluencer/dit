## Requirements
- [Rust + Cargo](https://www.rust-lang.org/tools/install)
- [IPFS](https://docs.ipfs.io/install/command-line/#package-managers)
- [Yew](https://yew.rs/docs/en/next/getting-started/project-setup)
- [Trunk](https://yew.rs/docs/en/next/getting-started/project-setup/using-trunk)

## Web-App
- Customize as needed.
- Change ENS name in the main.rs file.
- Compile with this command: trunk build --release
- Add and Pin the folder created by Trunk to IPFS. Command -> ```ipfs add --recursive --cid-version=1 FOLDER_NAME_HERE```
- Upload CID to Pinata Cloud, Temporal and/or host it yourself.
- (Optional) Upload CID to ENS or other DNS.

## Testing
### Setup 1
IPFS natively in Brave. (live streams won't work, cannot enable pubsub and IPNS is slow)
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
- Replace <INSERT_CID_HERE> with the root CID of your website.
- Restart browser

### Setup 2
IPFS + any browser
- [Install IPFS Desktop](https://docs.ipfs.io/install/ipfs-desktop/#ipfs-desktop)
- Right click on IPFS tray icon, under settings, check both Enable PubSub & Enable IPNS over PubSub.
- Allow CORS with these commands. (Replace <INSERT_CID_HERE> with root CID of your website)
    - ```ipfs config --json API.HTTPHeaders.Access-Control-Allow-Methods '["GET", "POST", "PUT"]'```
    - ```ipfs config --json API.HTTPHeaders.Access-Control-Allow-Origin '["http://localhost:5001", "http://127.0.0.1:5001", "https://webui.ipfs.io", "http://<INSERT_CID_HERE>.ipfs.localhost:8080"]'```

### Dev Build
Checkout defluencer.eth on Ropsten testnet for a live example.