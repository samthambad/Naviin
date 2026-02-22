use std::sync::{Arc, RwLock};
use rust_decimal::prelude::*;

use crate::providers::{ProviderRegistry, ProviderBuilder, ProviderError};
use crate::provider_config::ProvidersConfig;

/// Global live engine that holds active API clients.
/// It is wrapped in an RwLock to allow multiple threads to read (fetch prices)
/// simultaneously, while allowing a single thread to write (rebuild the registry)
/// when the configuration changes.
static REGISTRY: RwLock<Option<Arc<ProviderRegistry>>> = RwLock::new(None);

/// Global configuration state (the "blueprint") loaded from providers.toml.
/// Stores user preferences like enabled providers and their priority order.
static CONFIG: RwLock<Option<ProvidersConfig>> = RwLock::new(None);

/// Loads the provider configuration from providers.toml into the global CONFIG.
/// This is typically called lazily when the config is first accessed.
fn load_config() -> ProvidersConfig {
    let mut config_guard = CONFIG.write().unwrap();
    let config = ProvidersConfig::load();
    *config_guard = Some(config.clone());
    config
}

/// Retrieves the current global configuration.
/// If the configuration hasn't been loaded yet, it triggers load_config().
fn get_config() -> ProvidersConfig {
    {
        let config_guard = CONFIG.read().unwrap();
        if let Some(ref config) = *config_guard {
            return config.clone();
        }
    }
    load_config()
}

/// Builds a new ProviderRegistry engine based on the provided configuration blueprint.
/// It initializes the actual HTTP/WebSocket clients for each enabled provider.
fn build_registry(config: &ProvidersConfig) -> ProviderRegistry {
    let mut builder = ProviderBuilder::new();
    
    if config.is_provider_enabled("yahoo") {
        builder = builder.with_yahoo();
    }
    
    if config.is_provider_enabled("coingecko") {
        builder = builder.with_coingecko();
    }
    
    if config.is_provider_enabled("fmp")
        && let Some(key) = config.get_api_key("fmp") {
            builder = builder.with_fmp(&key);
        }
    
    if config.is_provider_enabled("twelve_data")
        && let Some(key) = config.get_api_key("twelve_data") {
            builder = builder.with_twelve_data(&key);
        }
    
    if config.is_provider_enabled("binance") {
        builder = builder.with_binance_stream();
    }
    
    // Apply the custom priority order defined in the config
    builder = builder.with_priority(config.to_priority());
    
    // Set the primary providers for stock and crypto
    if let Some(ref default) = config.default_stock {
        builder = builder.default_stock(default);
    }
    if let Some(ref default) = config.default_crypto {
        builder = builder.default_crypto(default);
    }
    
    builder.build()
}

/// Retrieves the global ProviderRegistry engine.
/// If the engine hasn't been built yet, it initializes it using the current config.
fn get_registry() -> Arc<ProviderRegistry> {
    {
        let registry_guard = REGISTRY.read().unwrap();
        if let Some(ref registry) = *registry_guard {
            return registry.clone();
        }
    }
    
    let config = get_config();
    let registry = Arc::new(build_registry(&config));
    
    {
        let mut registry_guard = REGISTRY.write().unwrap();
        *registry_guard = Some(registry.clone());
    }
    
    registry
}

/// Forces a reload of the configuration file and rebuilds the provider engine.
/// This is used to apply changes made at runtime (e.g., via TUI commands).
pub fn reload_providers() {
    let config = load_config();
    let registry = Arc::new(build_registry(&config));
    
    let mut registry_guard = REGISTRY.write().unwrap();
    *registry_guard = Some(registry);
}

/// Returns a thread-safe handle to the market data engine.
pub fn get_provider_registry() -> Arc<ProviderRegistry> {
    get_registry()
}

/// Returns a copy of the current provider settings.
pub fn get_providers_config() -> ProvidersConfig {
    get_config()
}

/// Saves the provided configuration to providers.toml and reloads the engine.
pub fn save_providers_config(config: &ProvidersConfig) -> Result<(), std::io::Error> {
    config.save()?;
    reload_providers();
    Ok(())
}

/// Enables a specific provider by name.
/// Validates if an API key is required and present before enabling.
pub fn enable_provider(name: &str) -> Result<(), String> {
    let mut config = get_config();
    
    if !["yahoo", "coingecko", "fmp", "twelve_data", "binance"].contains(&name) {
        return Err(format!("Unknown provider: {}", name));
    }
    
    let needs_key = ["fmp", "twelve_data"].contains(&name);
    if needs_key {
        let has_key = config.get_api_key(name).is_some();
        if !has_key {
            let env_var = match name {
                "fmp" => "FMP_API_KEY",
                "twelve_data" => "TWELVE_DATA_API_KEY",
                _ => "API_KEY",
            };
            return Err(format!(
                "Provider '{}' requires API key. Set {} environment variable.",
                name, env_var
            ));
        }
    }
    
    config.set_provider_enabled(name, true);
    save_providers_config(&config)
        .map_err(|e| format!("Failed to save config: {}", e))?;
    
    Ok(())
}

/// Disables a specific provider by name.
pub fn disable_provider(name: &str) -> Result<(), String> {
    let mut config = get_config();
    
    if name == "yahoo" {
        return Err("Cannot disable yahoo - it's the default provider".to_string());
    }
    
    config.set_provider_enabled(name, false);
    save_providers_config(&config)
        .map_err(|e| format!("Failed to save config: {}", e))?;
    
    Ok(())
}

/// Sets the primary provider for a specific instrument type (e.g., "stock" or "crypto").
pub fn set_default_provider(instrument_type: &str, provider: &str) -> Result<(), String> {
    let mut config = get_config();
    
    if !config.is_provider_enabled(provider) {
        return Err(format!("Provider '{}' is not enabled", provider));
    }
    
    match instrument_type.to_lowercase().as_str() {
        "stock" => {
            config.default_stock = Some(provider.to_string());
        }
        "crypto" => {
            config.default_crypto = Some(provider.to_string());
        }
        _ => {
            return Err(format!(
                "Unknown instrument type: {}. Use 'stock' or 'crypto'.",
                instrument_type
            ));
        }
    }
    
    save_providers_config(&config)
        .map_err(|e| format!("Failed to save config: {}", e))?;
    
    Ok(())
}

/// Configures the order in which providers are tried when fetching data for a specific type.
pub fn set_provider_priority(instrument_type: &str, providers: &[String]) -> Result<(), String> {
    let mut config = get_config();
    
    for provider in providers {
        if !config.is_provider_enabled(provider) {
            return Err(format!("Provider '{}' is not enabled", provider));
        }
    }
    
    match instrument_type.to_lowercase().as_str() {
        "stock" => {
            config.priority.stock = providers.to_vec();
        }
        "crypto" => {
            config.priority.crypto = providers.to_vec();
        }
        "forex" => {
            config.priority.forex = providers.to_vec();
        }
        "etf" => {
            config.priority.etf = providers.to_vec();
        }
        "index" => {
            config.priority.index = providers.to_vec();
        }
        _ => {
            return Err(format!(
                "Unknown instrument type: {}. Use stock, crypto, forex, etf, or index.",
                instrument_type
            ));
        }
    }
    
    save_providers_config(&config)
        .map_err(|e| format!("Failed to save config: {}", e))?;
    
    Ok(())
}

/// Returns a summary of all available providers and their current activation status.
pub fn list_provider_status() -> Vec<(String, bool, Option<String>)> {
    let config = get_config();
    let all_providers = vec![
        ("yahoo", None),
        ("coingecko", None),
        ("fmp", Some("FMP_API_KEY")),
        ("twelve_data", Some("TWELVE_DATA_API_KEY")),
        ("binance", None),
    ];
    
    all_providers
        .into_iter()
        .map(|(name, key_env)| {
            let enabled = config.is_provider_enabled(name);
            let has_key = key_env.map(|k| std::env::var(k).is_ok()).unwrap_or(true);
            let status = if enabled {
                if has_key {
                    Some("enabled".to_string())
                } else {
                    Some("needs key".to_string())
                }
            } else {
                None
            };
            (name.to_string(), enabled, status)
        })
        .collect()
}

/// Gateway function to fetch the previous day's closing price for a symbol.
/// Automatically handles provider selection and fallbacks.
pub async fn previous_price_close(symbol: &String, print: bool) -> Decimal {
    let registry = get_registry();
    
    match registry.get_previous_close(symbol).await {
        Ok(price) => {
            if print {
                println!("Previous close: {price}");
            }
            price
        }
        Err(_) => Decimal::ZERO,
    }
}

/// Gateway function to fetch the current market price for a symbol.
/// It uses a fallback strategy: if the primary provider fails (e.g., rate limited),
/// it automatically tries the next enabled provider in the priority list.
pub async fn curr_price(symbol: &String, print: bool) -> Decimal {
    let registry = get_registry();
    
    match registry.get_current_price_with_fallback(symbol).await {
        Ok(price) => {
            if print {
                println!("Current price: {}", price.value);
            }
            price.value
        }
        Err(_) => Decimal::ZERO,
    }
}

/// Fetches a detailed quote (open, high, low, change, etc.) for a symbol.
pub async fn get_quote(symbol: &str) -> Result<crate::providers::types::Quote, ProviderError> {
    let registry = get_registry();
    registry.get_quote(symbol).await
}

/// Fetches current prices for multiple symbols in parallel (if supported by provider) or sequence.
pub async fn get_batch_prices(symbols: &[&str]) -> Vec<(String, Result<Decimal, ProviderError>)> {
    let registry = get_registry();
    let results = registry.get_batch_prices(symbols).await;
    
    results.into_iter()
        .map(|(symbol, result)| {
            (symbol, result.map(|p| p.value))
        })
        .collect()
}

/// Lists names of all currently enabled market data providers.
pub fn list_available_providers() -> Vec<String> {
    let registry = get_registry();
    registry.available_providers().into_iter().map(|s| s.to_string()).collect()
}

/// Lists names of all currently enabled streaming data providers (WebSockets).
pub fn list_stream_providers() -> Vec<String> {
    let registry = get_registry();
    registry.available_stream_providers().into_iter().map(|s| s.to_string()).collect()
}
