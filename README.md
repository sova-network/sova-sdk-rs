# Sova SDK

Sova SDK is a Rust library for interacting with the Sova MEV Block Engine and Searcher services.
It provides functionalities for authentication, streaming mempool transactions, subscribing to bundles, and sending bundles.

## Features

- **Authentication**: Authenticate using your private key to obtain access and refresh tokens.
- **Streaming Mempool Transactions**: Stream transactions from the client to the Sova MEV Block Engine.
- **Subscribe to Bundles**: Subscribe to receive a stream of simulated and profitable bundles.
- **Send Bundles**: Send bundles to the Sova MEV Block Engine for processing.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
sova_sdk_rs = { git = "https://github.com/sova-network/sova-sdk-rs" }
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request with any changes or enhancements. Follow these steps to contribute:

1. Fork the repository.
2. Create a new branch for your feature or bugfix.
3. Make your changes and commit them with clear and descriptive messages.
4. Push your changes to your fork.
5. Open a pull request to the main repository.
