// SPDX-License-Identifier: MIT

use std::{
    path::Path,
    process::Command,
    sync::atomic::{AtomicU32, Ordering},
    thread,
    time::Duration,
};

static SESSION_COUNTER: AtomicU32 = AtomicU32::new(0);

struct TmuxSession {
    name: String,
}

impl TmuxSession {
    pub fn new(width: u16, height: u16) -> Self {
        let id = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
        let name = format!("tgrep_test_{}", id);
        let w = width.to_string();
        let h = height.to_string();
        Command::new("tmux")
            .args(["kill-session", "-t", &name])
            .output()
            .ok();
        let status = Command::new("tmux")
            .args(["new-session", "-d", "-s", &name, "-x", &w, "-y", &h])
            .status()
            .expect("failed to create tmux session");
        assert!(status.success(), "tmux new-session failed");
        let status = Command::new("tmux")
            .args(["resize-window", "-t", &name, "-x", &w, "-y", &h])
            .status()
            .expect("tmux resize-window failed");
        assert!(status.success(), "tmux resize-window failed");
        TmuxSession { name }
    }

    pub fn run_cmd(&self, cmd: &str) {
        Command::new("tmux")
            .args(["send-keys", "-t", &self.name, cmd, "Enter"])
            .status()
            .expect("tmux send-keys failed");
    }

    pub fn send_key(&self, key: &str) {
        Command::new("tmux")
            .args(["send-keys", "-t", &self.name, key])
            .status()
            .expect("tmux send-keys failed");
        thread::sleep(Duration::from_millis(150));
    }

    pub fn capture(&self) -> String {
        let out = Command::new("tmux")
            .args(["capture-pane", "-t", &self.name, "-p"])
            .output()
            .expect("tmux capture-pane failed");
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    fn wait_for_navigate_mode(&self) {
        for _ in 0..100 {
            thread::sleep(Duration::from_millis(50));
            let curr = self.capture();
            if curr.lines().any(|l| l.trim_start().starts_with("-> ")) {
                return;
            }
        }
        panic!(
            "tgrep never reached navigate mode (no '-> ' seen after 5s)\nfinal pane:\n{}",
            self.capture()
        );
    }

    pub fn wait_stable(&self) -> Vec<u8> {
        let mut prev = String::new();
        let mut stable_count = 0;
        for _ in 0..60 {
            thread::sleep(Duration::from_millis(50));
            let curr = self.capture();
            if !curr.trim().is_empty() {
                if curr == prev {
                    stable_count += 1;
                    if stable_count >= 3 {
                        return normalize(curr);
                    }
                } else {
                    stable_count = 0;
                    prev = curr;
                }
            }
        }
        normalize(prev)
    }
}

impl Drop for TmuxSession {
    fn drop(&mut self) {
        Command::new("tmux")
            .args(["kill-session", "-t", &self.name])
            .output()
            .ok();
    }
}

fn normalize(s: String) -> Vec<u8> {
    let lines: Vec<&str> = s.lines().map(|l| l.trim_end()).collect();
    let end = lines
        .iter()
        .rposition(|l| !l.is_empty())
        .map_or(0, |i| i + 1);
    lines[..end].join("\n").into_bytes()
}

fn tgrep_cmd(path: &Path, mode_args: &str) -> String {
    let cmd_path = env!("CARGO_BIN_EXE_tgrep");
    format!(
        "TREEGREP_DEFAULT_OPTS=\"\" {} --no-color --no-bold --threads=1 {} -p={} \
         --char-vertical='|' --char-horizontal=- --char-tee=+ --char-bottom-left=+ \
         --ellipsis='~' --selected-indicator=\"-> \"",
        cmd_path,
        mode_args,
        path.to_string_lossy()
    )
}

pub fn get_menu_output_from_search(path: &Path, keys: &[&str]) -> Vec<u8> {
    let session = TmuxSession::new(80, 24);
    session.run_cmd(&tgrep_cmd(path, "--menu"));
    session.wait_stable();

    for key in keys {
        session.send_key(key);
    }

    session.wait_stable()
}

pub fn get_menu_output(path: &Path, tgrep_args: &str, keys: &[&str]) -> Vec<u8> {
    let session = TmuxSession::new(80, 24);
    session.run_cmd(&tgrep_cmd(path, &format!("--select {}", tgrep_args)));
    session.wait_for_navigate_mode();

    for key in keys {
        session.send_key(key);
    }

    session.wait_stable()
}
