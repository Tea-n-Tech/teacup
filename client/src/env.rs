use std::env::{self};

const ENV_API_TOKEN: &str = "TEACUP_TOKEN";

pub fn get_api_token() -> String {
    env::var(ENV_API_TOKEN).expect("TEACUP_TOKEN env var is not set")
}
