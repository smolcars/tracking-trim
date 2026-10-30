//! Launch at login. The OS registration is the only state; the settings file stays out of it.

use std::io;

pub fn is_enabled() -> bool {
    #[cfg(windows)]
    return reg("query", &[]).is_ok();
    #[cfg(not(windows))]
    entry().is_ok_and(|path| path.exists())
}

pub fn enable() -> io::Result<()> {
    // Refused rather than lossy: a mangled path would register a binary that does not exist.
    let exe = std::env::current_exe()?.into_os_string().into_string();
    let exe = exe.map_err(|_| io::Error::other("executable path is not Unicode"))?;
    // The stored value keeps its own quotes, or Windows splits a path with spaces at login.
    #[cfg(windows)]
    return reg("add", &["/t", "REG_SZ", "/d", &format!("\"{exe}\""), "/f"]);
    // No launchctl on macOS: bootstrapping would start a second instance now.
    #[cfg(not(windows))]
    {
        let path = entry()?;
        std::fs::create_dir_all(path.parent().ok_or(io::ErrorKind::NotFound)?)?;
        #[cfg(target_os = "macos")]
        let text = plist(&exe);
        #[cfg(not(target_os = "macos"))]
        let text = desktop_entry(&exe);
        std::fs::write(path, text)
    }
}

pub fn disable() -> io::Result<()> {
    #[cfg(windows)]
    return reg("delete", &["/f"]);
    #[cfg(not(windows))]
    std::fs::remove_file(entry()?)
}

#[cfg(not(windows))]
fn entry() -> io::Result<std::path::PathBuf> {
    // Empty or relative values are unusable, same as settings.rs.
    let env = |key: &str| {
        std::env::var_os(key)
            .map(std::path::PathBuf::from)
            .filter(|p| p.is_absolute())
    };
    #[cfg(target_os = "macos")]
    let path =
        env("HOME").map(|home| home.join("Library/LaunchAgents/com.smolcars.tracking-trim.plist"));
    #[cfg(not(target_os = "macos"))]
    let path = env("XDG_CONFIG_HOME")
        .or_else(|| env("HOME").map(|home| home.join(".config")))
        .map(|config| config.join("autostart/tracking-trim.desktop"));
    path.ok_or(io::ErrorKind::NotFound.into())
}

#[cfg(any(test, target_os = "macos"))]
fn plist(exe: &str) -> String {
    let exe = exe
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>Label</key><string>com.smolcars.tracking-trim</string>
<key>ProgramArguments</key><array><string>{exe}</string></array>
<key>RunAtLoad</key><true/>
</dict></plist>
"#
    )
}

// Quoted so spaces survive; Exec's escaping backslash is doubled for the key file, and % starts a field code.
#[cfg(any(test, not(any(target_os = "macos", windows))))]
fn desktop_entry(exe: &str) -> String {
    let exe = exe
        .replace('\\', r"\\\\")
        .replace('"', r#"\\""#)
        .replace('`', r"\\`")
        .replace('$', r"\\$")
        .replace('%', "%%");
    format!("[Desktop Entry]\nType=Application\nName=tracking-trim\nExec=\"{exe}\"\n")
}

#[cfg(windows)]
fn reg(verb: &str, extra: &[&str]) -> io::Result<()> {
    use std::os::windows::process::CommandExt;
    const RUN: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
    // Absolute: a bare name also resolves next to this binary, so a planted reg.exe would run.
    let root = std::env::var_os("SystemRoot").unwrap_or_else(|| r"C:\Windows".into());
    let output = std::process::Command::new(std::path::Path::new(&root).join(r"System32\reg.exe"))
        .args([verb, RUN, "/v", "tracking-trim"])
        .args(extra)
        .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
        .output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("reg.exe {verb} failed")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plist_escapes_the_path() {
        assert!(plist("/R&D <x>/tt").contains("<string>/R&amp;D &lt;x&gt;/tt</string>"));
    }

    #[test]
    fn desktop_entry_quotes_and_escapes_the_path() {
        let text = desktop_entry(r#"/My Apps/50%/$"`\tt"#);
        assert!(text.starts_with("[Desktop Entry]\nType=Application\nName=tracking-trim\n"));
        assert!(text.contains(r#"Exec="/My Apps/50%%/\\$\\"\\`\\\\tt""#));
    }
}
