use std::path::Path;

use crate::models::{ShellConfig, ShellKind};

pub fn detect_shells() -> Vec<ShellConfig> {
    let mut shells = Vec::new();
    #[cfg(target_os = "windows")]
    {
        shells.push(ShellConfig::builtin(
            ShellKind::PowerShell,
            "builtin-powershell",
            "PowerShell",
            "powershell.exe",
            vec!["-NoProfile".into()],
        ));
        shells.push(ShellConfig::builtin(
            ShellKind::Cmd,
            "builtin-cmd",
            "CMD",
            "cmd.exe",
            vec![],
        ));
        if let Some(pwsh) = find_pwsh() {
            shells.push(ShellConfig::builtin(
                ShellKind::PowerShell,
                "builtin-pwsh",
                "PowerShell 7",
                &pwsh,
                vec!["-NoProfile".into()],
            ));
        }
        if let Some(bash) = find_git_bash() {
            shells.push(ShellConfig::builtin(
                ShellKind::Bash,
                "builtin-bash",
                "Git Bash",
                &bash,
                vec!["--login".into()],
            ));
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let bash = find_in_path("bash").unwrap_or_else(|| "/bin/bash".to_string());
        shells.push(ShellConfig::builtin(
            ShellKind::Bash,
            "builtin-bash",
            "Bash",
            &bash,
            vec!["-l".into()],
        ));
        shells.push(ShellConfig::builtin(
            ShellKind::Sh,
            "builtin-sh",
            "Sh",
            "sh",
            vec![],
        ));
        if let Some(pwsh) = find_in_path("pwsh") {
            shells.push(ShellConfig::builtin(
                ShellKind::PowerShell,
                "builtin-pwsh",
                "PowerShell 7",
                &pwsh,
                vec!["-NoProfile".into()],
            ));
        }
    }
    shells
}

fn find_in_path(exe: &str) -> Option<String> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let full = dir.join(exe);
        if full.is_file() {
            return Some(full.to_string_lossy().into_owned());
        }
        #[cfg(target_os = "windows")]
        {
            let with_exe = dir.join(format!("{exe}.exe"));
            if with_exe.is_file() {
                return Some(with_exe.to_string_lossy().into_owned());
            }
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn find_pwsh() -> Option<String> {
    if let Some(p) = find_in_path("pwsh.exe") {
        return Some(p);
    }
    if let Ok(pf) = std::env::var("PROGRAMFILES") {
        let cand = Path::new(&pf).join("PowerShell").join("7").join("pwsh.exe");
        if cand.is_file() {
            return Some(cand.to_string_lossy().into_owned());
        }
    }
    if let Some(la) = std::env::var_os("LOCALAPPDATA") {
        let cand = Path::new(&la)
            .join("Microsoft")
            .join("WindowsApps")
            .join("pwsh.exe");
        if cand.is_file() {
            return Some(cand.to_string_lossy().into_owned());
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn find_git_bash() -> Option<String> {
    for c in [
        r"C:\Program Files\Git\bin\bash.exe",
        r"C:\Program Files\Git\usr\bin\bash.exe",
    ] {
        if Path::new(c).is_file() {
            return Some(c.to_string());
        }
    }
    if let Some(la) = std::env::var_os("LOCALAPPDATA") {
        for c in [r"Programs\Git\bin\bash.exe", r"Programs\Git\usr\bin\bash.exe"] {
            let cand = Path::new(&la).join(c);
            if cand.is_file() {
                return Some(cand.to_string_lossy().into_owned());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_shells_returns_platform_builtins() {
        let shells = detect_shells();
        assert!(!shells.is_empty());
        assert!(shells.iter().all(|s| s.builtin));
        let kinds: Vec<ShellKind> = shells.iter().map(|s| s.kind).collect();
        #[cfg(target_os = "windows")]
        {
            assert!(kinds.contains(&ShellKind::PowerShell));
            assert!(kinds.contains(&ShellKind::Cmd));
        }
        #[cfg(not(target_os = "windows"))]
        {
            assert!(kinds.contains(&ShellKind::Bash));
            assert!(kinds.contains(&ShellKind::Sh));
        }
    }
}
