// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

interface IConfiguration {
    function name() external view returns (string memory);

    function startAutowiringSources() external;
}
