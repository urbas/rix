use std::fs::File;
use std::io::Write;

pub fn tmp_file(tmp_dir: &tempfile::TempDir, file_name: &str, contents: &str) -> String {
    let path = tmp_dir.path().join(file_name).to_str().unwrap().to_owned();
    let mut file = File::create(&path).unwrap();
    file.write_all(contents.as_bytes()).unwrap();
    return path;
}
