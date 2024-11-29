/// This crate serves to handle all the data available from the resources dir

pub fn get_resource_dir() -> String {
    match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(v) => v,
        Err(err) => panic!("Could not open cargo folder. {}", err),
    }
}

pub fn get_test_path(input: &str) -> String {
    std::path::Path::new(&get_resource_dir())
        .join(format!("resources/test/{}", &input))
        .to_str()
        .unwrap()
        .to_string()
}
