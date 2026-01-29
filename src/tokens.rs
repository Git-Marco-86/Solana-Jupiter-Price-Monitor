/**
 * v1.2.0 28/09/2025
 * Author: Marco Maffei
 *
 */

use std::{collections::HashMap, fs::File, io::BufReader, path::Path};
use serde::Deserialize;
use once_cell::sync::Lazy;


// Carica a runtime solo la prima volta
pub static TOKENS: Lazy<TokenMap> = Lazy::new(|| {
    load_token_map("./tokens/tokens.json")
        .expect("tokens.json mancante o errato")
});

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct TokenInfo {
    pub order:       u8,
    pub symbol:      String,
    pub full_name:   String,
    pub mint_addr:   String,
    pub market_addr: String,
    pub sell_addr: String,
    pub buy_addr: String,
}

impl TokenInfo {
    #[allow(dead_code)]
    pub fn variants() -> Vec<TokenInfo> {
        let mut v: Vec<TokenInfo> =
            TOKENS.values().cloned().collect();
        v.sort_by_key(|t| t.order);
        v
    }
}

pub type TokenMap = HashMap<String, TokenInfo>;

pub fn load_token_map<P: AsRef<Path>>(path: P) -> anyhow::Result<TokenMap> {
    let f = File::open(path)?;
    let reader = BufReader::new(f);
    let map: TokenMap = serde_json::from_reader(reader)?;
    Ok(map)
}