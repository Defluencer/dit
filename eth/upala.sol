// SPDX-License-Identifier: MIT
pragma solidity ^0.8.3;

contract UpalaGroup {
    address private owner;

    uint256 public score;
    uint256 private fee;

    //Map address to membership
    mapping(address => bool) public memberships;

    mapping(uint256 => bool) private usedNonces;

    constructor() payable {
        owner = msg.sender;
        fee = msg.value;
    }

    /// The caller address will be added as a member if the nonce & signature is valid, must also pay entree fee.
    function addMember(uint256 nonce, bytes memory signature) public payable {
        require(!memberships[msg.sender], "Must not be a member.");
        require(msg.value == fee, "Must pay exact fee.");
        require(!usedNonces[nonce], "Cannot use nonce twice");

        usedNonces[nonce] = true;

        // this recreates the message that was signed on the client
        bytes32 message =
            prefixed(keccak256(abi.encodePacked(msg.sender, nonce, this)));

        require(
            recoverSigner(message, signature) == owner,
            "Must be approved by owner"
        );

        memberships[msg.sender] = true;
    }

    /// The caller can transfer it's membership to another address.
    function transferMembership(address to) public {
        require(memberships[msg.sender], "Must be a member.");

        memberships[msg.sender] = false;
        memberships[to] = true;
    }

    ///The caller can "explode", getting the value of score transfered to his address.
    function burnBridge() public {
        require(memberships[msg.sender], "Must be a member.");
        require(
            address(this).balance >= score,
            "Must have enough in contract balance."
        );

        memberships[msg.sender] = false;

        payable(msg.sender).transfer(score);
    }

    ///The caller can leave the group, getting the value of fee transfered to his address.
    function leaveGroup() public {
        require(memberships[msg.sender], "Must be a member.");
        require(
            address(this).balance >= fee,
            "Must have enough in contract balance."
        );

        memberships[msg.sender] = false;

        payable(msg.sender).transfer(score);
    }

    //https://docs.soliditylang.org/en/v0.8.3/solidity-by-example.html#creating-and-verifying-signatures

    function recoverSigner(bytes32 message, bytes memory sig)
        internal
        pure
        returns (address)
    {
        (uint8 v, bytes32 r, bytes32 s) = splitSignature(sig);

        return ecrecover(message, v, r, s);
    }

    /// signature methods.
    function splitSignature(bytes memory sig)
        internal
        pure
        returns (
            uint8 v,
            bytes32 r,
            bytes32 s
        )
    {
        require(sig.length == 65);

        assembly {
            // first 32 bytes, after the length prefix.
            r := mload(add(sig, 32))
            // second 32 bytes.
            s := mload(add(sig, 64))
            // final byte (first byte of the next 32 bytes).
            v := byte(0, mload(add(sig, 96)))
        }

        return (v, r, s);
    }

    /// builds a prefixed hash to mimic the behavior of eth_sign.
    function prefixed(bytes32 hash) internal pure returns (bytes32) {
        return
            keccak256(
                abi.encodePacked("\x19Ethereum Signed Message:\n32", hash)
            );
    }
}
