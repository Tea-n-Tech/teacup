const ENV_DB_USERNAME: &str = "TEACUP_DB_USER";
const ENV_DB_PASSWORD: &str = "TEACUP_DB_PW";

pub fn get_db_username() -> String {
    tc_core::get_env_var_or_panic(ENV_DB_USERNAME)
}

pub fn get_db_password() -> String {
    tc_core::get_env_var_or_panic(ENV_DB_PASSWORD)
}
