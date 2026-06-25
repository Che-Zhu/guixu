use serial_test::serial;

use guixu::config::{load_config_from_env, ConfigError};

fn clear_env() {
    std::env::remove_var("GUIXU_BIND_ADDR");
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
        config.youzhiyouxing_cookie.expose_for_test(),
        "_weasley_key=abc123"
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
