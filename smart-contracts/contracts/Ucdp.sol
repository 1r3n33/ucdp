// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0;

contract Ucdp {
    struct Partner {
        bytes32 name;
        bool enabled;
        bool registered;
    }

    mapping(address => Partner) public partners;

    constructor() {
        // Insert a dummy partner for test.
        partners[address(0x123)] = Partner("partner", true, true);
    }

    function registerPartner(bytes32 name) external {
        require(
            partners[msg.sender].registered == false,
            "Sender already registered"
        );
        partners[msg.sender] = Partner(name, true, true);
    }
}
