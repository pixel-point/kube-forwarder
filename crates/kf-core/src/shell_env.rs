//! Importing the environment of the user's login shell.
//!
//! Applications started from Finder or the Dock get a minimal environment
//! (`PATH=/usr/bin:/bin:/usr/sbin:/sbin`). Credential plugins referenced by kube configs
//! (`aws`, `gcloud`, `kubelogin`, ...) usually live in `/opt/homebrew/bin` or similar and
//! often rely on variables such as `AWS_PROFILE`. Like the original application
//! (`shell-path`) and VS Code, the environment of a login shell is imported at startup.

use std::io::{IsTerminal, Read};
use std::process::{Command, Stdio};
use std::time::Duration;

const START: &str = "__KUBE_FORWARDER_ENV_START__";
const END: &str = "__KUBE_FORWARDER_ENV_END__";

/// Variables that describe the shell session itself and must not leak into the app.
const SKIPPED: &[&str] = &[
    "PWD",
    "OLDPWD",
    "SHLVL",
    "_",
    "TERM",
    "TERM_PROGRAM",
    "TERM_PROGRAM_VERSION",
    "TERM_SESSION_ID",
    "COLORTERM",
    "ITERM_SESSION_ID",
    "TMUX",
    "TMUX_PANE",
    "XPC_FLAGS",
    "XPC_SERVICE_NAME",
    "__CFBundleIdentifier",
    "SECURITYSESSIONID",
];

/// The environment is imported on macOS when the app was not started from a terminal.
pub fn should_import() -> bool {
    cfg!(target_os = "macos")
        && !std::io::stdin().is_terminal()
        && std::env::var_os("KUBE_FORWARDER_SKIP_SHELL_ENV").is_none()
}

/// Runs the login shell and returns its environment.
pub fn read_login_shell_env(timeout: Duration) -> Option<Vec<(String, String)>> {
    let shell = std::env::var("SHELL")
        .ok()
        .filter(|shell| !shell.is_empty())
        .unwrap_or_else(|| "/bin/zsh".into());
    let script = format!("printf '%s' '{START}'; /usr/bin/env -0; printf '%s' '{END}'");
    let mut child = Command::new(&shell)
        .args(["-l", "-i", "-c", &script])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut stdout = child.stdout.take()?;
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut output = Vec::new();
        let _ = stdout.read_to_end(&mut output);
        let _ = sender.send(output);
    });
    let output = match receiver.recv_timeout(timeout) {
        Ok(output) => output,
        Err(_) => {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
    };
    let _ = child.wait();
    parse_env_output(&output)
}

/// Extracts `KEY=VALUE` pairs printed by `env -0` between the markers.
pub fn parse_env_output(output: &[u8]) -> Option<Vec<(String, String)>> {
    let text = String::from_utf8_lossy(output);
    let start = text.find(START)? + START.len();
    let end = text[start..].find(END)? + start;
    Some(
        text[start..end]
            .split('\0')
            .filter_map(|entry| {
                let (key, value) = entry.split_once('=')?;
                let key = key.trim_start_matches('\n');
                (!key.is_empty() && !SKIPPED.contains(&key))
                    .then(|| (key.to_string(), value.to_string()))
            })
            .collect(),
    )
}

/// Imports the login shell environment into the current process.
///
/// # Safety
///
/// Calls [`std::env::set_var`]: it must run before any other thread is started.
pub unsafe fn import_login_shell_env() -> usize {
    if !should_import() {
        return 0;
    }
    let Some(vars) = read_login_shell_env(Duration::from_secs(5)) else {
        return 0;
    };
    for (key, value) in &vars {
        // SAFETY: the caller guarantees that the process is still single threaded.
        unsafe { std::env::set_var(key, value) };
    }
    vars.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_env_output_between_markers() {
        let output = format!(
            "Last login: today\n{START}PATH=/opt/homebrew/bin:/usr/bin\0AWS_PROFILE=dev\0PWD=/tmp\0EMPTY=\0WEIRD=a=b\0{END}motd"
        );
        let vars = parse_env_output(output.as_bytes()).unwrap();
        assert_eq!(
            vars,
            vec![
                ("PATH".to_string(), "/opt/homebrew/bin:/usr/bin".to_string()),
                ("AWS_PROFILE".to_string(), "dev".to_string()),
                ("EMPTY".to_string(), String::new()),
                ("WEIRD".to_string(), "a=b".to_string()),
            ]
        );
    }

    #[test]
    fn missing_markers() {
        assert!(parse_env_output(b"PATH=/usr/bin").is_none());
    }

    #[cfg(unix)]
    #[test]
    fn reads_a_real_shell() {
        let vars = read_login_shell_env(Duration::from_secs(20)).expect("login shell output");
        assert!(vars.iter().any(|(key, _)| key == "PATH"));
    }
}
