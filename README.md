# DeTrack Smart Contract (CosmWASM)

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

This is a CosmWASM smart contract for the DeTrack project on the c4e chain. It provides functionality for storing and verifying cryptographic proofs of energy data.

## Overview

The DeTrack smart contract implements a decentralized node management and proof verification system for energy data on the Chain4Energy blockchain.

**Current Version:** Phase 1b (DID-First Architecture - Variant C)  
**Target Version:** Phase 2 (Reputation-Based Architecture)  
**Status:** Roadmap planning phase



## Contract Structure

- `src/contract.rs` - Main contract entry points (instantiate, execute, query)
- `src/error.rs` - Error handling
- `src/execute.rs` - Execute message handlers
- `src/msg.rs` - Message definitions
- `src/query.rs` - Query handlers
- `src/state.rs` - State management



## Documentation

- [API Specification](./docs/api-specification.md) - Complete API reference
- [Contract Design](./docs/contract-design.md) - Design overview
- [Deployment Guide](./docs/deployment-guide.md) - Deployment instructions


## License

Apache-2.0
- Nodes must maintain a reputation score above the minimum threshold
- Only the admin can whitelist/remove nodes and update reputation scores

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to contribute to this project.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Support

For support and questions:
- Open an issue on [GitHub Issues](https://github.com/chain4energy/detrack-node-contract/issues)
- Visit the [Chain4Energy organization](https://github.com/chain4energy)

## Related Projects

- [DeTrack Worker Node](https://github.com/chain4energy/detrack-worker-node) - Node implementation for the DeTrack network
- [DID Contract](https://github.com/chain4energy/did-contract) - Decentralized Identity contract for c4e chain
