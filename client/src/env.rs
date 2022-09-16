const ENV_API_TOKEN: &str = "TEACUP_TOKEN";

pub fn get_api_token() -> String {
    tc_core::get_env_var_or_panic(ENV_API_TOKEN)
}
