## Requirements
- Rust + Cargo https://www.rust-lang.org/tools/install
- Yew https://yew.rs/docs/en/next/getting-started/project-setup
- Trunk https://yew.rs/docs/en/next/getting-started/project-setup/using-trunk

## Web-App
- Customize as needed.
- Compile with this command: trunk build --release
- Add and Pin the www folder to IPFS using this command: ipfs add --recursive --cid-version=1 www
- Upload CID to ENS or other DNS.
- Upload CID to Pinata Cloud, Temporal and/or host it yourself.

## Viewers
Only Brave browser include IPFS but it can't be configured with pubsub enabled so we are stuck with this annoying setup for now.
- Install IPFS CLI https://docs.ipfs.io/install/command-line/#package-managers
- Allow CORS with these commands:
    - ipfs config --json API.HTTPHeaders.Access-Control-Allow-Methods '["GET", "POST", "PUT"]'
    - ipfs config --json API.HTTPHeaders.Access-Control-Allow-Origin '["INSERT_YOUR_DOMAIN_HERE"]'
- Start IPFS with this command: ipfs daemon --enable-pubsub-experiment
- Launch any Browser.
- Navigate to domain.
- Enjoy!