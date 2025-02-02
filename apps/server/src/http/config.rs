use std::path::PathBuf;

fn get_dotenv_path() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".env")
        .to_str()
        .expect(".env is not found in the correct directory!")
        .to_string()
}

pub struct Env {
    pub client_url: String,
    pub server_url: String,
    pub database_url: String,
    pub jwt_secret: String,
    pub lsp_token: String,
}

impl Env {
    pub fn init() {
        dotenvy::from_path(get_dotenv_path()).expect(".env file not found");
    }

    pub fn get() -> Self {
        Self {
            client_url: std::env::var("CLIENT_URL").expect("CLIENT_URL is void"),
            server_url: std::env::var("SERVER_URL").expect("SERVER_URL is void"),
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL is void"),
            jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET is void"),
            lsp_token: std::env::var("LSP_TOKEN").expect("LSP_TOKEN is void"),
        }
    }
}
