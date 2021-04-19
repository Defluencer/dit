// SPDX-License-Identifier: MIT
pragma solidity ^0.8.3;

//https://docs.soliditylang.org/en/v0.8.3/solidity-by-example.html#creating-and-verifying-signatures

function recoverSigner(bytes32 message, bytes memory sig)
    pure
    returns (address)
{
    (uint8 v, bytes32 r, bytes32 s) = splitSignature(sig);

    return ecrecover(message, v, r, s);
}

/// signature methods.
function splitSignature(bytes memory sig)
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
function prefixed(bytes32 hash) pure returns (bytes32) {
    return
        keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", hash));
}
