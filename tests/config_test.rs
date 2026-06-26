use std::collections::HashMap;

use serial_test::serial;

use guixu::config::{load_config_from_env, AppConfig, ConfigError, SecretString};

fn clear_env() {
    std::env::remove_var("GUIXU_BIND_ADDR");
    std::env::remove_var("PORT");
    std::env::remove_var("YOUZHIYOUXING_COOKIE");
    std::env::remove_var("KIMI_CODING_PLAN_TOKEN");
}

fn load_config_from_env_with(
    overrides: &[(String, String)],
) -> Result<AppConfig, ConfigError> {
    let mut preserved: HashMap<String, Option<String>> = HashMap::new();
    for (key, _) in overrides {
        preserved.insert(key.clone(), std::env::var(key).ok());
    }

    for (key, value) in overrides {
        std::env::set_var(key, value);
    }

    let result = load_config_from_env();

    for (key, value) in preserved {
        match value {
            Some(v) => std::env::set_var(&key, v),
            None => std::env::remove_var(&key),
        }
    }

    result
}

#[test]
#[serial]
fn loads_required_youzhiyouxing_cookie() {
    clear_env();
    std::env::set_var("YOUZHIYOUXING_COOKIE", "_weasley_key=abc123");
    std::env::set_var("KIMI_CODING_PLAN_TOKEN", "sk-test-token");

    let config = load_config_from_env().expect("config should load");

    assert_eq!(config.bind_addr, "127.0.0.1:3000");
    assert_eq!(
        config.youzhiyouxing_cookie,
        SecretString::new("_weasley_key=abc123".to_string())
    );
}

#[test]
#[serial]
fn rejects_missing_youzhiyouxing_cookie() {
    clear_env();
    std::env::set_var("KIMI_CODING_PLAN_TOKEN", "sk-test-token");

    let error = load_config_from_env().expect_err("missing cookie should fail");

    assert_eq!(error, ConfigError::MissingEnv("YOUZHIYOUXING_COOKIE"));
}

#[test]
#[serial]
fn rejects_cookie_without_weasley_key_name() {
    clear_env();
    std::env::set_var("YOUZHIYOUXING_COOKIE", "abc123");
    std::env::set_var("KIMI_CODING_PLAN_TOKEN", "sk-test-token");

    let error = load_config_from_env().expect_err("raw cookie value should fail");

    assert_eq!(error, ConfigError::InvalidYouzhiyouxingCookie);
}

#[test]
#[serial]
fn binds_to_railway_port_when_port_env_is_set() {
    clear_env();
    std::env::set_var("PORT", "4321");
    std::env::set_var("YOUZHIYOUXING_COOKIE", "_weasley_key=abc123");
    std::env::set_var("KIMI_CODING_PLAN_TOKEN", "sk-test-token");

    let config = load_config_from_env().expect("config should load");

    assert_eq!(config.bind_addr, "0.0.0.0:4321");
}

#[test]
#[serial]
fn explicit_bind_addr_overrides_port_env() {
    clear_env();
    std::env::set_var("GUIXU_BIND_ADDR", "127.0.0.1:3009");
    std::env::set_var("PORT", "4321");
    std::env::set_var("YOUZHIYOUXING_COOKIE", "_weasley_key=abc123");
    std::env::set_var("KIMI_CODING_PLAN_TOKEN", "sk-test-token");

    let config = load_config_from_env().expect("config should load");

    assert_eq!(config.bind_addr, "127.0.0.1:3009");
}

#[test]
fn config_loads_kimi_coding_plan_token() {
    let token = "sk-kimi-test-token";
    let env = HashMap::from([
        ("GUIXU_BIND_ADDR", "127.0.0.1:3000"),
        ("YOUZHIYOUXING_COOKIE", "_weasley_key=test"),
        ("KIMI_CODING_PLAN_TOKEN", token),
    ]);

    let config = load_config_from_env_with(
        &env.into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<Vec<_>>(),
    )
    .expect("config should load");

    assert_eq!(config.kimi_coding_plan_token.as_str(), token);
}
