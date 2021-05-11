// SPDX-License-Identifier: MIT
pragma solidity ^0.8.3;

import "upala/signatures.sol";

contract UpalaGroup {
    address private owner;

    uint256 private oldScore;
    uint256 public score;

    uint256 private oldFee;
    uint256 public fee;

    //Map address to membership
    mapping(address => bool) public memberships;

    mapping(uint256 => bool) private usedNonces;

    uint256 public gracePeriodEnd;

    event NewMembership(address member);
    event BridgeBurned(address member);
    event LossMembership(address member);
    event ScoreChanged(uint256 newScore, uint256 gracePeriodEnd);
    event FeeChanged(uint256 newFee, uint256 gracePeriodEnd);

    constructor(uint256 startFee) payable {
        require(startFee < msg.value, "Fee must be lower than score");

        owner = msg.sender;
        score = msg.value;
        fee = startFee;
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
        emit NewMembership(msg.sender);
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

        uint256 amount = score;

        if (block.timestamp <= gracePeriodEnd && oldScore > score) {
            amount = oldScore;
        }

        require(
            address(this).balance >= amount,
            "Must have enough in contract balance."
        );

        memberships[msg.sender] = false;
        emit BridgeBurned(msg.sender);

        payable(msg.sender).transfer(amount);
    }

    ///The caller can leave the group, getting the value of fee transfered to his address.
    function leaveGroup() public {
        require(memberships[msg.sender], "Must be a member.");
        require(
            fee <= address(this).balance,
            "Must have enough in contract balance."
        );

        uint256 amount = fee;

        if (block.timestamp <= gracePeriodEnd && oldFee > fee) {
            amount = oldFee;
        }

        memberships[msg.sender] = false;
        emit LossMembership(msg.sender);

        payable(msg.sender).transfer(amount);
    }

    //  <-- Management Functions Below -->

    /// Allow the owner to change the score and trigger grace period.
    function changeScore(uint256 newScore) public {
        require(msg.sender == owner, "Must be the owner");
        require(
            block.timestamp > gracePeriodEnd,
            "Cannot update during grace period"
        );
        require(newScore > fee, "Score Must be higher than Fee");
        require(
            newScore <= address(this).balance,
            "Must have enough in contract balance."
        );

        gracePeriodEnd = block.timestamp + 86400; // ~ 1 day

        oldScore = score;
        score = newScore;

        emit ScoreChanged(newScore, gracePeriodEnd);
    }

    ///Allow the onwer to change the entree fee and trigger grace period.
    function changeFee(uint256 newFee) public {
        require(msg.sender == owner, "Must be the owner");
        require(
            block.timestamp > gracePeriodEnd,
            "Cannot update during grace period"
        );
        require(newFee < score, "Fee Must be lower than Score");

        gracePeriodEnd = block.timestamp + 86400; // ~ 1 day

        oldFee = fee;
        fee = newFee;

        emit FeeChanged(newFee, gracePeriodEnd);
    }
}
