use std::{env, fs, io, path::Path};

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
    let build_number = match fs::read_to_string(&file_path) {
        Ok(s) => s.parse::<i32>().unwrap(),
        Err(e) if e.kind() == io::ErrorKind::NotFound => 0,
        Err(e) => {
            eprintln!("{:#?}", e);
            panic!("cannot handle this error");
        }
    };
    fs::write(&file_path, (build_number + 1).to_string()).unwrap();
}
