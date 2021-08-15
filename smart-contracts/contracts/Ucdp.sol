// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0;

contract Ucdp {
    struct Partner {
        bytes32 name;
        bool enabled;
    }

    mapping(address => Partner) public partners;

    constructor() {
        partners[address(0x123)] = Partner("partner", true);
    }
}
