// SPDX-License-Identifier: MIT

use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};

static TEST_DIR: &str = "treegrep_tests";

pub struct Dir {
    pub path: PathBuf,
}

impl Dir {
    pub fn new(name: &str) -> Self {
        let path = env::temp_dir().join(TEST_DIR).join(name);
        if path.exists() {
            fs::remove_dir_all(&path).ok().unwrap();
        }
        fs::create_dir_all(&path).ok().unwrap();
        Dir { path }
    }

    pub fn add_child(&self, name: &PathBuf) {
        let p = self.path.join(name);
        if !p.exists() {
            fs::create_dir(p).ok().unwrap();
        }
    }

    pub fn create_file_str(&self, file_name: &PathBuf, content: &str) {
        self.create_file_bytes(file_name, content.as_bytes());
    }

    pub fn create_file_bytes(&self, file_name: &PathBuf, content: &[u8]) {
        let file_path = self.path.join(file_name);
        let mut file = fs::File::create(file_path).expect("Failed to create file");

        file.write_all(content).expect("Failed to write to file");
    }

    pub fn link_dir(&self, target_dir: &Path, link_name: &str) {
        let link_path = self.path.join(link_name);

        #[cfg(unix)]
        std::os::unix::fs::symlink(target_dir, link_path).expect("failed to link dir");

        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(target_dir, &link_path).expect("failed to link dir");
    }

    pub fn link_file(&self, target_file: &Path, link_name: &str) {
        let link_path = self.path.join(link_name);

        #[cfg(unix)]
        std::os::unix::fs::symlink(target_file, link_path).expect("failed to link file");

        #[cfg(windows)]
        std::os::windows::fs::symlink_file(target_file, &link_path).expect("failed to link file");
    }
}
