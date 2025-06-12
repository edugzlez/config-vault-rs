# config-vault

[![Crates.io](https://img.shields.io/crates/v/config-vault.svg)](https://crates.io/crates/config-vault)
[![Documentation](https://docs.rs/config-vault/badge.svg)](https://docs.rs/config-vault)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

An extension for the `config` crate that allows loading configurations from HashiCorp Vault.

## Features

- Integration with the `config` crate through a custom `VaultSource`
- Support for HashiCorp Vault's KV1 & KV2 engine
- Secure loading of secrets through Vault's REST API

## Installation

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
config-vault = "0.1.0"
config = "0.15.11" # The version compatible with config-vault
```

## Basic Usage

```rust
use config::{Config, ConfigError};
use config_vault::VaultSource;

fn load_config() -> Result<Config, ConfigError> {
    let vault_source = VaultSource::new(
        "http://127.0.0.1:8200".to_string(),  // Vault address
        "hvs.EXAMPLE_TOKEN".to_string(),      // Vault token
        "secret".to_string(),                 // KV mount name
        "dev".to_string(),        // Secret path
    );

    vault_source.set_kv_version(KvVersion::V1); // change kv_version to KV1 if required

    // Build configuration incorporating Vault and other sources
    Config::builder()
        .add_source(vault_source)
        // You can add other configuration sources
        // .add_source(config::File::with_name("config/default"))
        // .add_source(config::Environment::with_prefix("APP"))
        .build()
}

fn main() -> Result<(), ConfigError> {
    let config = load_config()?;
    
    // Use the configuration as usual
    let db_url = config.get_string("database.url")?;
    println!("Database URL: {}", db_url);
    
    Ok(())
}
```

## Documentation

For more information, check the [complete documentation](https://docs.rs/config-vault).

## Requirements

- Rust 1.60 or higher
- An accessible HashiCorp Vault server or compatible like [RustyVault](https://github.com/Tongsuo-Project/RustyVault)

## License

This project is licensed under the MIT License - see the LICENSE file for details.
