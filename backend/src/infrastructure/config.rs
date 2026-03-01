/// Application configuration loaded from environment variables.
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub openai_api_key: String,
    pub gemini_api_key: String,
    pub port: u16,
}
