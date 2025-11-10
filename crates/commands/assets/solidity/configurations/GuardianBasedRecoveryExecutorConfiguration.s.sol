// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.24;

import {ShortStrings, ShortString} from "@openzeppelin/contracts/utils/ShortStrings.sol";
import {TransparentUpgradeableProxy} from "@openzeppelin/contracts/proxy/transparent/TransparentUpgradeableProxy.sol";

import {Vm} from "forge-std/Vm.sol";
import {StdConfig} from "forge-std/StdConfig.sol";

import {Sources} from "src/scripts/reflections/di/libraries/Sources.s.sol";
import {IConfiguration} from "src/scripts/reflections/di/interfaces/IConfiguration.s.sol";
import {IWiringMechanism} from "src/scripts/reflections/di/interfaces/IWiringMechanism.s.sol";
import {StdConfigBasedWiring} from "src/scripts/reflections/di/wiring/StdConfigBasedWiring.s.sol";
import {TUPConfiguration} from "src/scripts/reflections/di/configurations/TUPConfiguration.s.sol";

import {GuardianBasedRecoveryExecutor} from "zksync-os/modules/contrib/GuardianBasedRecoveryExecutor.sol";

contract GuardianBasedRecoveryExecutorConfiguration is IConfiguration {
    using ShortStrings for ShortString;
    using Sources for Sources.Source;

    StdConfig private config;
    Vm private vm;
    address private admin;
    address private finalizer;
    address private submitter;

    constructor(
        Vm _vm,
        IWiringMechanism _configBasedWiringMechanism,
        address _admin,
        address _finalizer,
        address _submitter
    ) {
        config = StdConfigBasedWiring(address(_configBasedWiringMechanism))
            .getConfig();
        vm = _vm;
        admin = _admin;
        finalizer = _finalizer;
        submitter = _submitter;
    }

    function name() external view override returns (string memory) {
        return "GuardianBasedRecoveryExecutor Configuration";
    }

    function startAutowiringSources() external override {
        address webAuthnValidator = config
            .get(
                Sources.Source.TransparentUpgradeableProxy.getFullNicknamedName(
                    ShortStrings.toShortString(
                        Sources.Source.WebAuthnValidator.toString()
                    )
                )
            )
            .toAddress();
        address eoaKeyValidator = config
            .get(
                Sources.Source.TransparentUpgradeableProxy.getFullNicknamedName(
                    ShortStrings.toShortString(
                        Sources.Source.EOAKeyValidator.toString()
                    )
                )
            )
            .toAddress();

        vm.broadcast();
        address recoveryExecutorImpl = address(
            new GuardianBasedRecoveryExecutor(
                webAuthnValidator,
                eoaKeyValidator
            )
        );

        TUPConfiguration tupConfig = new TUPConfiguration(
            vm,
            config,
            admin,
            recoveryExecutorImpl,
            Sources.Source.GuardianBasedRecoveryExecutor
        );
        bytes memory initData = abi.encodeCall(
            GuardianBasedRecoveryExecutor.initialize,
            (admin, finalizer, submitter)
        );
        tupConfig.setInitializationData(initData);
        tupConfig.startAutowiringSources();
    }
}
