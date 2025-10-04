use toml::Value;

pub fn inject_env_vars(val: &mut Value) {
    match val {
        Value::String(s) => {
            if s.starts_with("${") && s.ends_with("}") {
                let key = &s[2..s.len() - 1];
                if let Ok(env_val) = std::env::var(key) {
                    *s = env_val;
                }
            }
        }
        Value::Table(map) => {
            for (_, v) in map.iter_mut() {
                inject_env_vars(v);
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                inject_env_vars(v);
            }
        }
        _ => {}
    }
}

pub fn detect_env() -> String {
    if let Ok(env) = std::env::var("EXUM_ENV") {
        return env;
    }

    if cfg!(debug_assertions) {
        "dev".to_string()
    } else {
        "prod".to_string()
    }
}