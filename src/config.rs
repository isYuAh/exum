use std::{fs, sync::OnceLock};

use serde::Deserialize;
use toml::Value;

use crate::env::{detect_env, inject_env_vars};

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ApplicationConfig {
  pub addr: [u8; 4],
  pub port: u16,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            addr: [0, 0, 0, 0],
            port: 8080,
        }
    }
}

impl ApplicationConfig {
    pub fn from_file(path: &str) -> Self {
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|_| {
                eprintln!("⚠️  Failed to read config file `{}`. Using default.", path);
                String::new()
            });

        toml::from_str(&content).unwrap_or_else(|_| Self::default())
    }

    pub fn load() -> Self {
        let value = get_config();
        toml::Value::try_into(value.clone()).unwrap_or_else(|_| Self::default())
    }
}

fn merge_toml(base: &mut Value, other: &Value) {
    match (base, other) {
        (Value::Table(base_map), Value::Table(other_map)) => {
            for (k, v) in other_map {
                match base_map.get_mut(k) {
                    Some(bv) => merge_toml(bv, v),
                    None => {
                        base_map.insert(k.clone(), v.clone());
                    }
                }
            }
        }
        (b, o) => *b = o.clone(),
    }
}

static _EXUM_CONFIG: OnceLock<Value> = OnceLock::new();
pub fn get_config() -> &'static Value {
    _EXUM_CONFIG.get_or_init(|| load_config())
}

pub fn load_config() -> Value {
    let base_str = fs::read_to_string("config.toml").unwrap_or_default();
    let mut base: Value = toml::from_str(&base_str).unwrap_or(Value::Table(Default::default()));

    let env = detect_env();
    let env_file = format!("config.{env}.toml");
    if let Ok(env_str) = fs::read_to_string(&env_file) {
        if let Ok(env_val) = toml::from_str::<Value>(&env_str) {
            merge_toml(&mut base, &env_val);
        }
    }

    inject_env_vars(&mut base);
    base
}

pub fn get_value<T: serde::de::DeserializeOwned>(root: &Value, path: &str) -> Option<T> {
    let mut current = root;
    for seg in path.split('.') {
        current = current.get(seg)?;
    }
    toml::Value::try_into(current.clone()).ok()
}