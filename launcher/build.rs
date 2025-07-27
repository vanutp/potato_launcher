use std::env;
use std::fs;

fn main() {
    let build_envs = ["LAUNCHER_NAME", "VERSION_MANIFEST_URL"];

    let optional_envs = ["AUTO_UPDATE_BASE", "VERSION"];

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = format!("{out_dir}/generated.rs");

    let mut config_content = String::new();
    for env in build_envs.iter() {
        let value = env::var(env).unwrap_or_else(|_| panic!("{env} is not set"));
        config_content.push_str(&format!("pub const {env}: &str = \"{value}\";\n"));
    }
    for env in optional_envs.iter() {
        match env::var(env) {
            Ok(value) => {
                config_content.push_str(&format!(
                    "pub const {env}: Option<&str> = Some(\"{value}\");\n"
                ));
            }
            Err(_) => {
                config_content.push_str(&format!("pub const {env}: Option<&str> = None;\n"));
            }
        }
    }
    let use_native_glfw_default = env::var("USE_NATIVE_GLFW_DEFAULT")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .expect("USE_NATIVE_GLFW_DEFAULT must be a boolean");
    config_content.push_str(&format!(
        "pub const USE_NATIVE_GLFW_DEFAULT: bool = {use_native_glfw_default};\n"
    ));
    fs::write(dest_path, config_content).unwrap();

    let data_launcher_name = env::var("LAUNCHER_NAME")
        .unwrap()
        .to_lowercase()
        .replace(" ", "_");
    let mut res = winres::WindowsResource::new();

    if cfg!(target_os = "windows") {
        res.set_icon(&format!("assets/{data_launcher_name}.ico"));
        res.compile().unwrap();
    }

    let icon_src = format!(
        "{}/assets/{}.png",
        env::var("CARGO_MANIFEST_DIR").unwrap().replace("\\", "/"),
        data_launcher_name
    );
    let icon_out = format!("{out_dir}/icon_file_bytes.rs");
    fs::write(
        &icon_out,
        format!("pub const LAUNCHER_ICON: &[u8] = include_bytes!(\"{icon_src}\");"),
    )
    .unwrap();
}
