// SPDX-License-Identifier: MIT
pragma solidity ^0.8.3;

import "upala/signatures.sol";

contract UpalaGroups {
    uint256 private nextId;

    struct Group {
        address owner;
        uint256 balance;
        uint256 oldScore;
        uint256 score;
        uint256 oldFee;
        uint256 fee;
        mapping(address => bool) memberships;
        mapping(uint256 => bool) usedNonces;
        uint256 gracePeriodEnd;
    }

    mapping(uint256 => Group) private groups;
    mapping(address => uint256[]) private userGroups;

    event ScoreChanged(
        uint256 groupId,
        uint256 newScore,
        uint256 gracePeriodEnd
    );
    event FeeChanged(uint256 groupId, uint256 newFee, uint256 gracePeriodEnd);

    /// The caller address will be added as a member if the nonce & signature is valid, must also pay entree fee.
    function joinGroup(
        uint256 groupId,
        uint256 nonce,
        bytes memory signature
    ) public payable {
        Group storage group = groups[groupId];

        require(!group.memberships[msg.sender], "Must not be a member.");
        require(msg.value == group.fee, "Must pay exact fee.");
        require(!group.usedNonces[nonce], "Cannot use nonce twice");

        group.usedNonces[nonce] = true;

        // this recreates the message that was signed on the client
        bytes32 message =
            prefixed(
                keccak256(abi.encodePacked(msg.sender, nonce, groupId, this))
            );

        require(
            recoverSigner(message, signature) == group.owner,
            "Must be approved by owner"
        );

        group.memberships[msg.sender] = true;
        userGroups[msg.sender].push(groupId);

        group.balance += msg.value;
    }

    /// The caller can transfer it's membership to another address.
    function transferMembership(uint256 groupId, address to) public {
        Group storage group = groups[groupId];

        require(group.memberships[msg.sender], "Must be a member.");

        group.memberships[msg.sender] = false;
        group.memberships[to] = true;
    }

    ///The caller can "explode", getting the value of his cumulative score transfered to his address.
    function burnBridge() public {
        uint256[] storage groupIds = userGroups[msg.sender];
        uint256 amount = 0;

        for (uint16 i = 0; i < groupIds.length; i++) {
            Group storage group = groups[groupIds[i]];

            if (!group.memberships[msg.sender]) {
                continue;
            }

            uint256 payout = group.score;

            if (
                block.timestamp <= group.gracePeriodEnd &&
                group.oldScore > payout
            ) {
                payout = group.oldScore;
            }

            if (group.balance < payout) {
                continue;
            }

            group.memberships[msg.sender] = false;
            amount += payout;
        }

        require(amount > 0);

        //don't clear userGroups because it cost gas for no reason???

        payable(msg.sender).transfer(amount);
    }

    ///The caller can leave his group, getting the value of fee transfered to his address.
    function leaveGroup(uint256 groupId) public {
        Group storage group = groups[groupId];

        require(group.memberships[msg.sender], "Must be a member.");
        require(group.fee <= group.balance, "Must have enough in balance.");

        uint256 amount = group.fee;

        if (
            block.timestamp <= group.gracePeriodEnd && group.oldFee > group.fee
        ) {
            amount = group.oldFee;
        }

        group.memberships[msg.sender] = false;

        payable(msg.sender).transfer(amount);
    }

    //  <-- Management Functions Below -->

    function createGroup(uint256 startFee) public payable returns (uint256) {
        require(startFee < msg.value, "Fee must be lower than score");

        uint256 groupId = nextId;
        nextId++;

        Group storage newGroup = groups[groupId];

        //Should the owner be a member???

        newGroup.owner = msg.sender;
        newGroup.balance = msg.value;
        newGroup.score = msg.value;
        newGroup.fee = startFee;

        return groupId;
    }

    /// Allow the owner to change the score and trigger grace period.
    function changeScore(uint256 groupId, uint256 newScore) public {
        Group storage group = groups[groupId];

        require(msg.sender == group.owner, "Must be the owner");
        require(
            block.timestamp > group.gracePeriodEnd,
            "Cannot update during grace period"
        );
        require(newScore > group.fee, "Score Must be higher than Fee");
        require(
            newScore <= group.balance,
            "Must have enough in contract balance."
        );

        group.gracePeriodEnd = block.timestamp + 86400; // ~ 1 day

        group.oldScore = group.score;
        group.score = newScore;

        emit ScoreChanged(groupId, newScore, group.gracePeriodEnd);
    }

    ///Allow the owner to change the entree fee and trigger grace period.
    function changeFee(uint256 groupId, uint256 newFee) public {
        Group storage group = groups[groupId];

        require(msg.sender == group.owner, "Must be the owner");
        require(
            block.timestamp > group.gracePeriodEnd,
            "Cannot update during grace period"
        );
        require(newFee < group.score, "Fee Must be lower than Score");

        group.gracePeriodEnd = block.timestamp + 86400; // ~ 1 day

        group.oldFee = group.fee;
        group.fee = newFee;

        emit FeeChanged(groupId, newFee, group.gracePeriodEnd);
    }

    /// Allow the owner to add a member to the group, must pay entree fee.
    function addMember(uint256 groupId, address newMember) public payable {
        Group storage group = groups[groupId];

        require(msg.sender == group.owner, "Must be the owner");
        require(
            block.timestamp > group.gracePeriodEnd,
            "Cannot update during grace period"
        );
        require(!group.memberships[newMember], "Must not be a member");
        require(msg.value == group.fee, "Must pay exact fee.");

        group.memberships[msg.sender] = true;
        userGroups[msg.sender].push(groupId);

        group.balance += msg.value;
    }
}
