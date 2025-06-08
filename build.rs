use std::{env, fs, path::Path};

fn main() {
    let opt_level = env::var_os("OPT_LEVEL").unwrap();
    let root_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let file_path = Path::new(&root_dir).join({
        if opt_level == "3" {
            "BUILD_NUMBER_RELEASE.txt"
        } else {
            "BUILD_NUMBER.txt"
        }
    });
    let build_number = fs::read_to_string(&file_path)
        .unwrap()
        .parse::<i32>()
        .unwrap();
    fs::write(&file_path, (build_number + 1).to_string()).unwrap();
}
