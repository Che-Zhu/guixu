use serial_test::serial;

use guixu::config::{load_config_from_env, ConfigError};

fn clear_env() {
    std::env::remove_var("GUIXU_BIND_ADDR");
    std::env::remove_var("PORT");
    std::env::remove_var("YOUZHIYOUXING_COOKIE");
}

#[test]
#[serial]
fn loads_required_youzhiyouxing_cookie() {
    clear_env();
    std::env::set_var("YOUZHIYOUXING_COOKIE", "_weasley_key=abc123");

    let config = load_config_from_env().expect("config should load");

    assert_eq!(config.bind_addr, "127.0.0.1:3000");
    assert_eq!(
        config.youzhiyouxing_cookie,
        guixu::config::SecretString::new("_weasley_key=abc123".to_string())
    );
}

#[test]
#[serial]
fn rejects_missing_youzhiyouxing_cookie() {
    clear_env();

    let error = load_config_from_env().expect_err("missing cookie should fail");

    assert_eq!(error, ConfigError::MissingEnv("YOUZHIYOUXING_COOKIE"));
}

#[test]
#[serial]
fn rejects_cookie_without_weasley_key_name() {
    clear_env();
    std::env::set_var("YOUZHIYOUXING_COOKIE", "abc123");

    let error = load_config_from_env().expect_err("raw cookie value should fail");

    assert_eq!(error, ConfigError::InvalidYouzhiyouxingCookie);
}

#[test]
#[serial]
fn binds_to_railway_port_when_port_env_is_set() {
    clear_env();
    std::env::set_var("PORT", "4321");
    std::env::set_var("YOUZHIYOUXING_COOKIE", "_weasley_key=abc123");

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

    let config = load_config_from_env().expect("config should load");

    assert_eq!(config.bind_addr, "127.0.0.1:3009");
}
