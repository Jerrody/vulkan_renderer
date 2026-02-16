use std::fs;

pub fn load_shader(path: &str) -> Vec<u8> {
    fs::read(path).unwrap()
}
