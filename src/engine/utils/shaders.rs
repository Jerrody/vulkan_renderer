use std::fs;

pub fn load_shader(path: &str) -> Vec<u8> {
    let data = fs::read(path).unwrap();

    data
}
