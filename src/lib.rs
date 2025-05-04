//! # config-vault
//!
//! `config-vault` is an extension for the `config` crate that allows loading configurations
//! directly from HashiCorp Vault.
//!
//! This library implements a custom `Source` for the `config` crate that can
//! connect to a HashiCorp Vault server and load secrets from the KV2 engine as
//! configuration values.
//!
//! ## Example
//!
//! ```
//! use config::{Config, ConfigError};
//! use config_vault::VaultSource;
//!
//! fn load_config() -> Result<Config, ConfigError> {
//!     let vault_source = VaultSource::new(
//!         "http://127.0.0.1:8200".to_string(),  // Vault address
//!         "hvs.EXAMPLE_TOKEN".to_string(),      // Vault token
//!         "secret".to_string(),                 // KV mount name
//!         "dev".to_string(),        // Secret path
//!     );
//!
//!     Config::builder()
//!         .add_source(vault_source)
//!         // You can add other sources
//!         .build()
//! }
//! ```

use std::collections::HashMap;

use config::{ConfigError, Map, Source, Value};
use reqwest::blocking::Client;
use serde_json::Value as JsonValue;
use url::Url;

/// A `Source` for the `config` library that loads configurations from HashiCorp Vault.
///
/// This source connects to a HashiCorp Vault server and loads a secret from
/// the version 2 of the KV (Key-Value) engine. The values from the secret are included
/// in the configuration as flat key-value pairs.
///
/// # Example
///
/// ```
/// use config_vault::VaultSource;
///
/// let vault = VaultSource::new(
///     "http://vault.example.com:8200".to_string(),
///     "my-token".to_string(),
///     "secret".to_string(),
///     "dev".to_string()
/// );
/// ```
#[derive(Debug, Clone)]
pub struct VaultSource {
    vault_addr: String,
    vault_token: String,
    vault_mount: String,
    vault_path: String,
}

impl VaultSource {
    /// Creates a new instance of `VaultSource`.
    ///
    /// # Parameters
    ///
    /// * `vault_addr` - Complete URL of the Vault server (e.g. "http://127.0.0.1:8200")
    /// * `vault_token` - Authentication token for Vault
    /// * `vault_mount` - Name of the KV engine mount (e.g. "secret")
    /// * `vault_path` - Path to the secret within the mount (e.g. "dev")
    ///
    /// # Example
    ///
    /// ```
    /// use config_vault::VaultSource;
    ///
    /// let source = VaultSource::new(
    ///     "http://127.0.0.1:8200".to_string(),
    ///     "hvs.EXAMPLE_TOKEN".to_string(),
    ///     "secret".to_string(),
    ///     "dev".to_string()
    /// );
    /// ```
    pub fn new(
        vault_addr: String,
        vault_token: String,
        vault_mount: String,
        vault_path: String,
    ) -> Self {
        Self {
            vault_addr,
            vault_token,
            vault_mount,
            vault_path,
        }
    }

    /// Builds the URL for Vault's KV2 engine read API.
    ///
    /// This function takes the base address of Vault and builds the complete URL
    /// to access the read API of the KV2 engine with the specified path.
    ///
    /// # Returns
    ///
    /// * `Result<Url, ConfigError>` - The constructed URL or an error if the address is invalid
    fn build_kv2_read_url(&self) -> Result<Url, ConfigError> {
        let api_path = format!("v1/{}/data/{}", self.vault_mount, self.vault_path);

        let mut url = Url::parse(&self.vault_addr)
            .map_err(|e| ConfigError::Message(format!("Invalid Vault address URL: {}", e)))?;

        url.path_segments_mut()
            .map_err(|_| ConfigError::Message("Vault address URL cannot be a base".into()))?
            .pop_if_empty() // Remove trailing slash if any
            .extend(api_path.split('/')); // Add the API path segments

        Ok(url)
    }
}

impl Source for VaultSource {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new(self.clone())
    }

    /// Implementation of the `collect` method from `Source`.
    ///
    /// This method makes an HTTP request to the Vault API to obtain
    /// configuration values stored in the specified secret.
    ///
    /// # Returns
    ///
    /// * `Result<Map<String, Value>, ConfigError>` - A map with configuration values
    ///   or an error if the request fails or the response format is not as expected.
    fn collect(&self) -> Result<Map<String, Value>, ConfigError> {
        let url = self.build_kv2_read_url()?;
        let client = Client::new();
        let response = client
            .get(url)
            .header("X-Vault-Token", &self.vault_token)
            .send()
            .map_err(|e| ConfigError::Foreign(Box::new(e)))?;

        if response.status().is_success() {
            let raw = response
                .json::<JsonValue>()
                .map_err(|e| ConfigError::Foreign(Box::new(e)))?;

            let json_obj = raw
                .get("data")
                .and_then(|x| x.get("data"))
                .and_then(|x| x.as_object())
                .unwrap();

            let mut secret = HashMap::new();
            for (k, v) in json_obj {
                secret.insert(k.clone(), Value::from(v.as_str().unwrap()));
            }

            Ok(secret)
        } else {
            Err(ConfigError::Message(format!(
                "Failed to fetch secret from Vault: {}",
                response.status()
            )))
        }
    }
}
