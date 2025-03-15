use std::sync::OnceLock;

pub struct AppConfig {
    pub api_base_url: String,
}

static APP_CONFIG: OnceLock<AppConfig> = OnceLock::new();

pub fn get_config() -> &'static AppConfig {
    APP_CONFIG.get_or_init(|| {
        // Default for local development
        #[cfg(debug_assertions)]
        let api_base = option_env!("API_BASE").unwrap_or("http://localhost:8000/friends");
        
        // Production default
        #[cfg(not(debug_assertions))]
        let api_base = option_env!("API_BASE").unwrap_or("http://64.181.233.1/friends");

        AppConfig {
            api_base_url: api_base.to_string(),
        }
    })
}