# Solidity Reflections ![Rust][rust-badge] [![License: MIT][license-badge]][license]

[rust-badge]: https://img.shields.io/badge/Built%20with%20-Rust-e43716.svg
[license]: https://opensource.org/licenses/MIT
[license-badge]: https://img.shields.io/badge/License-MIT-blue.svg

**Reflections** is a powerful command-line tool for Foundry projects that brings Java-style Reflection and Dependency Injection patterns to Solidity development. Built in Rust for performance and reliability.

## What is Reflections?

Reflections provides four core features:

1. **DI Framework Scaffolding** - Automatically scaffolds a complete Dependency Injection framework into your Foundry projects, enabling:
   - Automated contract deployment and wiring
   - Configuration-based deployment strategies
   - Named contract instances with deterministic addresses
   - Clean separation between deployment logic and contract code

2. **Contract Analysis** - Generates reflection libraries from your Solidity contracts:
   - Auto-generates `Sources.s.sol` enum from your contracts
   - Provides helper functions for contract metadata
   - Enables type-safe contract references in deployment scripts

3. **ABI Collection & Grouping** - Collects and organizes ABIs from Forge build artifacts:
   - Automatically discovers ABIs from compiled contracts
   - Groups contracts by NatSpec tags (@custom:swagger, @title, @notice)
   - Generates structured JSON output for easy consumption
   - Perfect for frontend integration and API documentation

4. **Interactive API Documentation** - Serves Swagger UI for your contract ABIs:
   - Auto-generates OpenAPI 3.0 specifications from ABIs
   - Interactive Swagger UI for exploring contract functions
   - View functions as GET endpoints, state-changing functions as POST
   - Perfect for developer documentation and API exploration

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

**DI Framework Setup:**
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

**ABI Documentation Workflow:**
```bash
# 1. Build your contracts
forge build

# 2. Collect ABIs with NatSpec grouping
reflections collect --output api/abis.json

# 3. Serve interactive Swagger UI
reflections serve --input api/abis.json --port 3000

# 4. Open http://localhost:3000 in your browser
# Explore your contract APIs with interactive documentation
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

### `reflections collect`

Collects ABIs from Forge build artifacts and groups them by NatSpec tags:

```bash
reflections collect [OPTIONS]

Options:
  -a, --artifacts-dir <DIR>  Forge build output directory [default: out]
  -o, --output <FILE>        Output JSON file [default: abis.json]
  -t, --tag <TAG>           NatSpec tag for grouping [default: @custom:swagger]
```

**Output format:**
```json
{
  "grouped": {
    "Core": [
      {
        "contract_name": "Counter",
        "file_path": "Counter.sol/Counter.json",
        "abi": [...],
        "group": "Core"
      }
    ],
    "Tokens": [...]
  },
  "ungrouped": [...]
}
```

**Supported tags:**
- `@custom:swagger` - Custom swagger grouping tag (default)
- `@title` - Contract title from NatSpec
- `@notice` - Contract notice from NatSpec
- Any `@custom:*` tag from devdoc/userdoc

**Example usage:**
```bash
# Build your contracts first
forge build

# Collect ABIs grouped by @custom:swagger tag
reflections collect

# Group by contract title
reflections collect --tag @title

# Custom output location
reflections collect --output build/abis.json
```

### `reflections serve`

Serves a Swagger UI interface for your collected ABIs with auto-generated OpenAPI documentation:

```bash
reflections serve [OPTIONS]

Options:
  -i, --input <FILE>   Path to collected ABIs JSON file [default: abis.json]
  -p, --port <PORT>    Port to serve on [default: 3000]
  --host <HOST>        Host to bind to [default: 127.0.0.1]
```

**Features:**
- Interactive Swagger UI at http://localhost:3000
- Auto-generated OpenAPI 3.0 specification
- View functions → GET endpoints with query parameters
- State-changing functions → POST endpoints with JSON bodies
- Grouped contracts organized by NatSpec tags

**Example usage:**
```bash
# First, build and collect ABIs
forge build
reflections collect

# Serve Swagger UI on default port 3000
reflections serve

# Custom port
reflections serve --port 8080

# Use different ABIs file
reflections serve --input build/abis.json --port 4000
```

## Understanding the Collect Command

The `collect` command is designed to bridge the gap between your Solidity contracts and frontend applications by extracting and organizing ABIs from Forge build artifacts.

### How It Works

1. **Discovers Artifacts**: Recursively scans your Forge output directory (default: `out/`) for compiled JSON artifacts
2. **Extracts ABIs**: Parses each artifact to extract the contract's ABI and metadata
3. **Groups by NatSpec**: Uses NatSpec documentation tags to organize contracts into logical groups
4. **Outputs JSON**: Generates a structured JSON file with grouped and ungrouped contracts

### NatSpec-Based Grouping

Add NatSpec tags to your Solidity contracts to control grouping:

```solidity
/// @custom:swagger Core
/// @title Counter Contract
/// @notice A simple counter implementation
contract Counter {
    uint256 public count;
    
    function increment() public {
        count++;
    }
}
```

The `@custom:swagger` tag (or any tag you specify with `--tag`) determines which group the contract belongs to in the output JSON and how it's organized in Swagger UI.

### Use Cases

**Frontend Integration**:
```bash
# Generate ABIs for your dApp
forge build && reflections collect --output src/abis.json
```

**API Documentation**:
```bash
# Group contracts by feature for documentation
reflections collect --tag @title --output docs/api/contracts.json
```

**Multi-Project Workflows**:
```bash
# Collect ABIs from a specific build
reflections collect --artifacts-dir target/out --output dist/abis.json
```

### Output Structure

The generated JSON provides both grouped and ungrouped contracts for maximum flexibility:

- **`grouped`**: Contracts organized by their NatSpec tag (e.g., "Core", "Tokens", "Utilities")
- **`ungrouped`**: Contracts without a NatSpec tag (still useful, just not categorized)

Each contract entry includes:
- `contract_name`: The contract's name
- `file_path`: Relative path to the artifact file
- `abi`: Full ABI array
- `group`: The group name (for grouped contracts only)

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
- [Test Coverage Report](TEST_COVERAGE.md) - Tests coverage stats
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
