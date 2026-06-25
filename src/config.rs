#[derive(Clone, PartialEq, Eq)]
pub struct SecretString(String);

impl SecretString {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for SecretString {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SecretString([redacted])")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppConfig {
    pub bind_addr: String,
    pub youzhiyouxing_cookie: SecretString,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ConfigError {
    #[error("missing required env var: {0}")]
    MissingEnv(&'static str),
    #[error("YOUZHIYOUXING_COOKIE must include _weasley_key=...")]
    InvalidYouzhiyouxingCookie,
}

pub fn load_config_from_env() -> Result<AppConfig, ConfigError> {
    let bind_addr =
        std::env::var("GUIXU_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let youzhiyouxing_cookie = std::env::var("YOUZHIYOUXING_COOKIE")
        .map_err(|_| ConfigError::MissingEnv("YOUZHIYOUXING_COOKIE"))?;

    if !youzhiyouxing_cookie
        .split(';')
        .any(|pair| pair.trim_start().starts_with("_weasley_key="))
    {
        return Err(ConfigError::InvalidYouzhiyouxingCookie);
    }

    Ok(AppConfig {
        bind_addr,
        youzhiyouxing_cookie: SecretString::new(youzhiyouxing_cookie),
    })
}
