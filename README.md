# Solidity Reflections API ![Rust][rust-badge] [![License: MIT][license-badge]][license]

[rust-badge]: https://img.shields.io/badge/Built%20with%20-Rust-e43716.svg
[license]: https://opensource.org/licenses/MIT
[license-badge]: https://img.shields.io/badge/License-MIT-blue.svg

**Reflections** is a powerful command-line tool that brings Java-style Reflection and Dependency Injection patterns to Solidity development. Built in Rust for performance and reliability.

## What is Reflections?

Reflections provides two core features:

1. **DI Framework Scaffolding** - Automatically scaffolds a complete Dependency Injection framework into your Foundry projects, enabling:
   - Automated contract deployment and wiring
   - Configuration-based deployment strategies
   - Named contract instances with deterministic addresses
   - Clean separation between deployment logic and contract code

2. **Contract Analysis** - Generates reflection libraries from your Solidity contracts:
   - Auto-generates `Sources.s.sol` enum from your contracts
   - Provides helper functions for contract metadata
   - Enables type-safe contract references in deployment scripts

### Installation

Install Reflections using Cargo:

```bash
cargo install reflections
```

Verify installation:

```bash
reflections --version
```

### Basic Usage

Initialize a Foundry project with the DI framework:

```bash
cd your-foundry-project
reflections init
```

Generate the Sources library from your contracts:

```bash
reflections generate
```

### Example Workflow

```bash
# 1. Initialize DI framework in your project
reflections init --openzeppelin-version v5.1.0

# 2. Generate Sources library from your contracts
reflections generate --contracts-dir src --output scripts/reflections/di/libraries/Sources.s.sol

# 3. Write deployment scripts extending Autowirable
# See scripts/reflections/di/README.md for detailed examples

# 4. Run your deployment with Foundry
forge script script/Deploy.s.sol --rpc-url $RPC --broadcast
```

## Core Commands

### `reflections init`

Scaffolds the complete DI framework into your project at `scripts/reflections/di/`:

```bash
reflections init [OPTIONS]

Options:
  --clean                           Remove previous scaffolding before re-initializing
  --openzeppelin-version <VERSION>  OpenZeppelin version [default: v5.1.0]
  --zksync-os-url <URL>            zkSync-OS repository URL
```

**What gets scaffolded:**
- `Autowirable.s.sol` - Base contract for deployment scripts
- `interfaces/` - Core DI interfaces
- `wiring/` - Wiring mechanism implementations  
- `configurations/` - Pre-built configuration contracts
- `.gitignoreTemplate` - Recommended gitignore entries
- `README.md` - Complete DI framework documentation

### `reflections generate`

Generates a reflection library from your Solidity contracts:

```bash
reflections generate [OPTIONS]

Options:
  -c, --contracts-dir <DIR>      Contracts directory [default: src]
  -o, --output <FILE>            Output file [default: scripts/reflections/di/libraries/Sources.s.sol]
  --library-name <NAME>          Library name [default: Sources]
  --license <SPDX>               SPDX license [default: MIT]
  --solidity-version <VERSION>   Solidity pragma [default: ^0.8.0]
```

**Generates:**
- Enum with all discovered contracts
- `toCreationCode()` - Get contract bytecode
- `toString()` - Get contract name as string
- `toSalt()` - Generate deterministic salts for CREATE2

## Configuration

Reflections uses `reflections.toml` for project configuration:

```toml
openzeppelin-version = "v5.1.0"
zksync-os-url = "https://github.com/matter-labs/zksync-os"
```

This file is automatically created/updated when running `reflections init`.

## Import Path Remapping

Reflections respects your `remappings.txt` file. During `reflections init`, all framework imports are automatically rewritten based on your project's remappings:

```txt
# Example remappings.txt
@openzeppelin/=lib/openzeppelin-contracts/
forge-std/=lib/forge-std/src/
zksync-os/=lib/zksync-os/contracts/
src/=src/
```

If no `remappings.txt` exists, default mappings are used to keep paths relative.

## DI Framework Features

The scaffolded DI framework provides:

- **Autowiring Modifiers**: `autowire`, `proxywire`, `configwire`, `accountwire`, `nickwire`
- **Configuration System**: Environment-based deployment configuration (debug/production)
- **Named Instances**: Deploy multiple instances of the same contract with unique nicknames
- **Proxy Support**: Built-in TransparentUpgradeableProxy deployment
- **Composition**: Combine multiple configurations into complex deployment flows

See `scripts/reflections/di/README.md` (after running `init`) for complete documentation and examples.

## Compile from Source

Clone the repository and build:

```bash
git clone https://github.com/GoldenSylph/solidity-reflections.git
cd solidity-reflections
cargo build --release
```

The binary will be located at `target/release/reflections`.

## Documentation

- [Usage Guide](USAGE.md) - Detailed usage examples
- [Changelog](CHANGELOG.md) - Version history and changes
- [Contributing Guide](CONTRIBUTING.md) - How to contribute
- [DI Framework Docs](crates/commands/assets/README.md) - Complete DI framework reference

## Example: Simple Deployment Script

```solidity
// script/Deploy.s.sol
pragma solidity ^0.8.24;

import {Autowirable} from "src/scripts/reflections/di/Autowirable.s.sol";
import {Sources} from "src/scripts/reflections/di/libraries/Sources.s.sol";

contract Deploy is Autowirable {
    function run() 
        public 
        autowire(Sources.Source.MyContract)
        proxywire(Sources.Source.MyToken)
    {
        // Contracts are now deployed and accessible
        address myContract = autowired(Sources.Source.MyContract);
        address myTokenProxy = autowired(
            Sources.Source.TransparentUpgradeableProxy,
            Sources.Source.MyToken.toSalt()
        );
        
        console.log("MyContract:", myContract);
        console.log("MyToken Proxy:", myTokenProxy);
    }
}
```

## Requirements

- Rust 1.75+ (for building from source)
- Foundry (for Solidity projects using the DI framework)

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Credits

Reflections was forked from [Soldeer](https://github.com/mario-eth/soldeer) and repurposed as a Solidity DI framework scaffolder and analysis tool.
