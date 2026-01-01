# Copilot Instructions for CosmWASM Smart Contracts

These guidelines instruct GitHub Copilot (the AI code assistant) on how to contribute effectively to **CosmWASM Smart Contract** projects for the Cosmos ecosystem.

## Project Overview

This is a template for **CosmWASM Smart Contracts** that typically:
- Implement decentralized logic on Cosmos-based blockchains
- Handle contract instantiation, execution, and queries
- Manage state through key-value storage
- Implement role-based access control and permissions
- Provide JSON schema definitions for messages
- Include comprehensive testing and deployment scripts

**Common Use Cases**: Token contracts, DeFi protocols, governance systems, staking mechanisms, data verification systems, and cross-chain applications.

## Project Structure

Use the following structure as a reference when adding or modifying files:

```
smart-contract/
├── .github/
│   ├── copilot-instructions.md  # This file
│   └── workflows/               # CI/CD configurations
├── docs/                        # Contract documentation
│   ├── contract-design.md       # High-level design document
│   ├── api-specification.md     # Message and query specification
│   └── deployment-guide.md      # Deployment instructions
├── src/
│   ├── lib.rs                   # Library entry point and module declarations
│   ├── contract.rs              # Main contract entry points (instantiate, execute, query)
│   ├── msg.rs                   # Message type definitions
│   ├── state.rs                 # State storage definitions
│   ├── execute.rs               # Execute message handlers
│   ├── query.rs                 # Query handlers
│   ├── error.rs                 # Custom error types
│   ├── helpers.rs               # Utility functions and helpers
│   ├── tests.rs                 # Unit tests (optional)
│   └── bin/
│       └── schema.rs            # Schema generation binary
├── schema/                      # Generated JSON schemas
│   ├── instantiate_msg.json
│   ├── execute_msg.json
│   ├── query_msg.json
│   └── config.json
├── scripts/                     # Deployment and testing scripts
│   ├── deploy_contract.sh       # Contract deployment script
│   ├── test-contract.sh         # Contract testing script
│   └── optimize.sh              # WASM optimization script
├── artifacts/                   # Compiled WASM binaries
│   ├── checksums.txt
│   └── contract.wasm
├── config/                      # Configuration files
│   ├── .env                     # Environment variables (NEVER commit!)
│   └── .env.example             # Template for configuration
├── .gitignore                   # Files to ignore in Git
├── Cargo.toml                   # Rust dependencies and metadata
├── Cargo.lock                   # Locked dependency versions
└── README.md                    # Project documentation
```

## Coding Guidelines

- **Style & Formatting**
  - Follow Rust best practices and conventions from the [Rust Style Guide](https://github.com/rust-dev-tools/fmt-rfcs/blob/master/guide/guide.md).
  - Use `cargo fmt` for consistent code formatting.
  - Use `cargo clippy` for linting and best practice recommendations.
  - Enable `#![deny(warnings)]` or similar strict compiler settings when appropriate.
  - Max file length: 800 lines. Split large files into smaller modules.
  - Prefer explicit error handling over `unwrap()` or `expect()` in production code.

- **Naming Conventions**
  - Files and modules: snake_case (e.g., `contract.rs`, `execute.rs`, `query.rs`).
  - Functions and variables: snake_case (e.g., `store_proof`, `query_config`, `node_address`).
  - Types, structs, and enums: PascalCase (e.g., `InstantiateMsg`, `ContractError`, `Config`).
  - Constants: UPPER_SNAKE_CASE (e.g., `CONTRACT_NAME`, `CONTRACT_VERSION`).
  - Private struct fields: Consider using snake_case with leading underscore for unused fields.

- **CosmWASM Best Practices**
  - Use `#[entry_point]` attribute for main contract functions (instantiate, execute, query, migrate).
  - Implement proper error handling with custom `ContractError` types using `thiserror`.
  - Use `cw-storage-plus` for efficient state management (Item, Map, IndexedMap).
  - Leverage `cosmwasm-schema` for automatic JSON schema generation.
  - Use `cw_serde` macro for serialization instead of manual serde derives.
  - Implement proper access control and permission checks.
  - Handle native token transfers and staking operations carefully.
  - Use `deps.api.addr_validate()` for address validation.
  - Implement proper migration logic when contract upgrades are needed.

- **State Management**
  - Define state storage clearly in `state.rs` using `Item` and `Map` from `cw-storage-plus`.
  - Use appropriate key types for maps (prefer `&Addr` over `String` for addresses).
  - Implement proper indexing for complex queries using `IndexedMap`.
  - Consider storage efficiency and gas costs when designing state structures.
  - Use `Config` struct pattern for global contract configuration.
  - Implement proper data validation before storing in state.

- **Message Design**
  - Separate message types: `InstantiateMsg`, `ExecuteMsg`, `QueryMsg`, `MigrateMsg`.
  - Use enum variants for different execute and query operations.
  - Implement role-based message separation (e.g., `AdminExecuteMsg`, `UserExecuteMsg`).
  - Use appropriate data types (`Uint128`, `Addr`, `Timestamp`) from `cosmwasm-std`.
  - Add proper JSON schema annotations and documentation.
  - Validate message parameters in handlers before processing.

- **Error Handling**
  - Create custom error types in `error.rs` using `thiserror`.
  - Use descriptive error messages that help with debugging.
  - Handle all possible error conditions explicitly.
  - Return appropriate HTTP status codes through error types.
  - Log errors appropriately for debugging (but avoid excessive logging).

- **Documentation**
  - Use English for all comments and documentation.
  - Use **Rust doc comments** (`///`) for public APIs, structs, and functions.
  - Document message types, state structures, and error conditions.
  - Include usage examples in documentation.
  - Maintain up-to-date API specification documentation.
  - Document gas usage patterns and optimization considerations.
  - Document migration procedures and breaking changes.

  Example:
  ```rust
  /// Stores a cryptographic proof on the blockchain.
  ///
  /// # Arguments
  /// * `deps` - Mutable dependencies for storage and API access
  /// * `env` - Environment information (block height, time, etc.)
  /// * `info` - Message information (sender, funds)
  /// * `data_hash` - SHA-256 hash of the verified data
  /// * `metadata` - Additional metadata to store with the proof
  ///
  /// # Returns
  /// * `Result<Response, ContractError>` - Success response or contract error
  ///
  /// # Errors
  /// * `ContractError::Unauthorized` - If sender lacks permission
  /// * `ContractError::InvalidHash` - If data_hash format is invalid
  pub fn store_proof(
      deps: DepsMut,
      env: Env,
      info: MessageInfo,
      data_hash: String,
      metadata: Option<String>,
  ) -> Result<Response, ContractError> {
      // implementation
  }
  ```

- **Testing**
  - Use **Rust's built-in testing framework** with `#[cfg(test)]` modules.
  - Use **cw-multi-test** for integration testing that simulates blockchain environment.
  - Test all execute and query handlers thoroughly.
  - Test error conditions and edge cases.
  - Test access control and permission systems.
  - Test state transitions and invariants.
  - Mock external dependencies when necessary.
  - Aim for high code coverage (>80%).

- **Security**
  - Validate all inputs thoroughly before processing.
  - Implement proper access control checks.
  - Handle integer overflow/underflow safely using checked arithmetic.
  - Validate addresses using `deps.api.addr_validate()`.
  - Be careful with native token handling and prevent reentrancy attacks.
  - Implement proper slashing and penalty mechanisms where applicable.
  - Review all state changes for potential manipulation vectors.

## Dependency Management

- **Cargo.toml Configuration**
  ```toml
  [package]
  name = "my-contract"
  version = "0.1.0"
  edition = "2021"
  
  [lib]
  crate-type = ["cdylib", "rlib"]
  
  [features]
  backtraces = ["cosmwasm-std/backtraces"]
  library = []
  
  [dependencies]
  cosmwasm-schema = "1.2"
  cosmwasm-std = "1.5"
  cosmwasm-storage = "1.2"
  cw-storage-plus = "1.0"
  cw2 = "1.0"
  schemars = "0.8"
  serde = { version = "1.0", default-features = false, features = ["derive"] }
  thiserror = "1.0"
  
  [dev-dependencies]
  cw-multi-test = "0.13"
  ```

- **Key Dependencies**
  - **cosmwasm-std**: Core CosmWASM standard library
  - **cosmwasm-schema**: JSON schema generation
  - **cw-storage-plus**: Advanced storage helpers
  - **cw2**: Contract version management
  - **serde**: Serialization/deserialization
  - **thiserror**: Error handling
  - **cw-multi-test**: Testing framework

- Always use compatible versions across cosmwasm dependencies
- Pin dependency versions in production contracts
- Regularly audit dependencies for security vulnerabilities
- Use `cargo audit` to check for known security issues

## Build & Optimization

- **Development Build**: `cargo build`
- **Release Build**: `cargo build --release --target wasm32-unknown-unknown`
- **Schema Generation**: `cargo run --bin schema`
- **Testing**: `cargo test`
- **Linting**: `cargo clippy -- -D warnings`
- **Formatting**: `cargo fmt`

- **WASM Optimization**
  ```bash
  # Using rust-optimizer Docker image
  docker run --rm -v "$(pwd)":/code \
    --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    cosmwasm/rust-optimizer:0.14.0
  ```

- **Production Checklist**
  - Run `cargo clippy` with no warnings
  - Ensure all tests pass with `cargo test`
  - Generate optimized WASM with rust-optimizer
  - Verify schema files are up to date
  - Check binary size is reasonable (<2MB typically)
  - Validate against cosmwasm-check if available

## Testing Strategy

- **Unit Tests**: Test individual functions and modules in isolation
- **Integration Tests**: Use `cw-multi-test` to test full contract workflows
- **Property Tests**: Test contract invariants and edge cases
- **Gas Tests**: Measure and optimize gas usage for expensive operations
- **Fuzz Testing**: Test with random inputs to find edge cases

Example test structure:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    
    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        
        let msg = InstantiateMsg {
            admin: None,
            // ... other fields
        };
        
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(res.messages.len(), 0);
    }
    
    #[test]
    fn test_unauthorized_access() {
        // Test access control
    }
}
```

## Schema Management

- **Generate schemas**: `cargo run --bin schema`
- Keep schemas in sync with message definitions
- Use schemas for frontend integration and documentation
- Version schemas alongside contract versions
- Validate generated schemas for completeness

## Deployment & Migration

- **Environment Setup**: Use `.env` files for network configurations
- **Deployment Scripts**: Automate deployment with shell scripts
- **Migration Planning**: Plan contract upgrades carefully
- **State Migration**: Handle state transitions during upgrades
- **Rollback Strategy**: Always have a rollback plan for failed deployments

## Security Guidelines

- **Input Validation**: Validate all user inputs and external data
- **Access Control**: Implement proper permission systems
- **Reentrancy Protection**: Protect against reentrancy attacks
- **Integer Safety**: Use checked arithmetic for financial operations
- **State Consistency**: Ensure state transitions maintain contract invariants
- **Audit Process**: Consider third-party security audits for production contracts

## Gas Optimization

- **Storage Efficiency**: Minimize storage operations and use efficient data structures
- **Computation Optimization**: Reduce expensive operations in loops
- **Message Batching**: Allow batch operations where appropriate
- **Lazy Loading**: Load only necessary data from storage
- **Pagination**: Implement pagination for large data sets

## Copilot Interactions

- **Context Awareness**: Review existing patterns in `contract.rs`, `execute.rs`, and `query.rs` before generating new code.
- **CosmWASM Conventions**: Follow established CosmWASM patterns for entry points, state management, and error handling.
- **Security First**: Always consider security implications and implement proper validation.
- **Type Safety**: Leverage Rust's type system for correctness and safety.
- **Testing**: Provide comprehensive tests for new functionality.
- **Documentation**: Include proper documentation for public APIs and complex logic.
- **Gas Awareness**: Consider gas costs and optimization in implementation decisions.
- **Migration Compatibility**: Consider upgrade paths and backward compatibility when modifying state structures.

## Example Contract Structure

```rust
// lib.rs
pub mod contract;
pub mod error;
pub mod execute;
pub mod helpers;
pub mod msg;
pub mod query;
pub mod state;

#[cfg(test)]
mod tests;

pub use crate::error::ContractError;

// contract.rs
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Implementation
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // Route to specific handlers
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    // Route to specific query handlers
}
```

This structure provides a solid foundation for building secure, efficient, and maintainable CosmWASM smart contracts.