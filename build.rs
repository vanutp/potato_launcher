use std::env;
use std::fs;

fn main() {
    let build_envs = [
        "LAUNCHER_NAME",
        "SERVER_BASE",
    ];

    let optional_envs = [
        "TGAUTH_BASE",
        "ELYBY_APP_NAME",
        "ELYBY_CLIENT_ID",
        "ELYBY_CLIENT_SECRET",
    ];

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = format!("{}/generated.rs", out_dir);

    let mut config_content = String::new();
    for env in build_envs.iter() {
        let value = env::var(env).expect(&format!("{} is not set", env));
        config_content.push_str(&format!("pub const {}: &str = \"{}\";\n", env, value));
    }
    for env in optional_envs.iter() {
        let value = env::var(env).unwrap_or_default();
        config_content.push_str(&format!("pub const {}: Option<&str> = Some(\"{}\");\n", env, value));
    }
    fs::write(dest_path, config_content).unwrap();
}