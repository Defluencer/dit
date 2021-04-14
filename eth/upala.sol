// SPDX-License-Identifier: MIT
pragma solidity ^0.8.3;

import "eth/signatures.sol";

contract UpalaGroup {
    address private owner;

    uint256 public score;
    uint256 private fee;

    //Map address to membership
    mapping(address => bool) public memberships;

    mapping(uint256 => bool) private usedNonces;

    uint256 public gracePeriodEnd;

    //TODO events???

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

    //  <-- Management Functions Below -->

    /// Allow the owner to change the score and trigger 1 hour grace period to protect bots from front-running attack.
    function changeScore(uint256 new_score) public {
        require(msg.sender == owner, "Must be the owner");
        require(
            block.timestamp >= gracePeriodEnd,
            "Cannot update during grace period"
        );
        require(new_score > fee, "Score Must be higher than Fee");

        gracePeriodEnd = block.timestamp + 3600; // ~ 1 hour
        score = new_score;
    }
}
