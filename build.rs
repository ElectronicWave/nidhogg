use std::{env, fs, path::PathBuf};

const RESOURCE_DIR: &str = "assets/authlib-injector";
const RESOURCES: [&str; 2] = ["latest.json", "latest.jar"];

fn main() {
    for resource in RESOURCES {
        println!("cargo:rerun-if-changed={RESOURCE_DIR}/{resource}");
    }

    if env::var_os("CARGO_FEATURE_AUTHLIB_INJECTOR_EMBED").is_none() {
        return;
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    for resource in RESOURCES {
        let source = PathBuf::from(format!("{RESOURCE_DIR}/{resource}"));
        let target = out_dir.join(resource);
        fs::copy(&source, &target).unwrap_or_else(|error| {
            panic!(
                "failed to copy '{}' to '{}': {error}",
                source.display(),
                target.display()
            )
        });
    }
}
