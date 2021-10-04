// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0;

contract Ucdp {
    struct Partner {
        bytes32 name;
        bool enabled;
        bool registered;
    }

    struct User {
        bytes32 name;
        bool registered;
    }

    mapping(address => Partner) public partners;

    mapping(address => User) public users;

    // Users give authorization to partners to use their data.
    // see function authorizePartner(address partner)
    mapping(address => mapping(address => bool)) public authorizedPartnersByUser;

    constructor() {
        // Insert a dummy partner for test.
        partners[address(0x123)] = Partner("partner", true, true);

        // Insert a dummy user that authorized the dummy partner to use its data.
        users[address(0x456)] = User("user auth", true);
        authorizedPartnersByUser[address(0x456)][address(0x123)] = true;

        // Insert a dummy user with no partner authorization.
        users[address(0x789)] = User("user no-auth", true);
    }

    function registerPartner(bytes32 name) external {
        require(
            partners[msg.sender].registered == false,
            "Sender already registered as a Partner"
        );
        partners[msg.sender] = Partner(name, true, true);
    }

    function registerUser(bytes32 name) external {
        require(
            users[msg.sender].registered == false,
            "Sender already registered as a User"
        );
        users[msg.sender] = User(name, true);
    }

    function authorizePartner(address partner) external {
        require(
            users[msg.sender].registered == true,
            "Sender must be registered as a User"
        );
        require(
            partners[partner].registered == true,
            "Partner must be registered"
        );
        authorizedPartnersByUser[msg.sender][partner] = true;
    }

    function unauthorizePartner(address partner) external {
        require(
            users[msg.sender].registered == true,
            "Sender must be registered as a User"
        );
        require(
            partners[partner].registered == true,
            "Partner must be registered"
        );
        authorizedPartnersByUser[msg.sender][partner] = false;
    }
}
