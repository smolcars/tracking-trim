use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[rustfmt::skip]
#[derive(Clone, Copy)]
pub enum Site { Instagram, X, YouTube, TikTok, Threads, Reddit }

impl Site {
    // Menu labels in declaration order, so `site as usize` indexes this and `Settings::sites`.
    // Lowercased, they are the file keys.
    pub const NAMES: [&str; 6] = ["Instagram", "X", "YouTube", "TikTok", "Threads", "Reddit"];
}

#[derive(Clone)]
pub struct Settings {
    pub sites: [bool; Site::NAMES.len()],
    pub fixupx: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            sites: [true; Site::NAMES.len()],
            fixupx: false,
        }
    }
}

impl Settings {
    pub fn load() -> Settings {
        app_dir().map_or_else(Settings::default, |dir| Settings::read(&dir))
    }

    pub fn save(&self) -> io::Result<()> {
        self.write(&app_dir().ok_or(io::ErrorKind::NotFound)?)
    }

    fn read(dir: &Path) -> Settings {
        let mut settings = Settings::default();
        let text = fs::read_to_string(dir.join("settings")).unwrap_or_default();
        for (key, value) in text.lines().filter_map(|line| line.split_once('=')) {
            let on = match value {
                "1" => true,
                "0" => false,
                _ => continue,
            };
            if key == "fixupx" {
                settings.fixupx = on;
            } else if let Some(i) = Site::NAMES.iter().position(|n| n.to_lowercase() == key) {
                settings.sites[i] = on;
            }
        }
        settings
    }

    fn write(&self, dir: &Path) -> io::Result<()> {
        let mut text = String::new();
        for (name, on) in Site::NAMES.iter().zip(self.sites) {
            text += &format!("{}={}\n", name.to_lowercase(), on as u8);
        }
        text += &format!("fixupx={}\n", self.fixupx as u8);
        fs::create_dir_all(dir)?;
        fs::write(dir.join("settings"), text)
    }
}

fn app_dir() -> Option<PathBuf> {
    // Empty or relative values are unusable, and XDG says to ignore them too.
    let env = |key: &str| {
        std::env::var_os(key)
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
    };
    #[cfg(target_os = "macos")]
    let base = env("HOME").map(|home| home.join("Library/Application Support"));
    #[cfg(windows)]
    let base = env("APPDATA");
    #[cfg(not(any(target_os = "macos", windows)))]
    let base = env("XDG_CONFIG_HOME").or_else(|| env("HOME").map(|home| home.join(".config")));
    Some(base?.join("tracking-trim"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_falls_back_to_defaults() {
        let dir = std::env::temp_dir().join(format!("tracking-trim-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        let missing = Settings::read(&dir);
        assert_eq!(missing.sites, [true; 6]);
        assert!(!missing.fixupx);

        let mut settings = Settings::default();
        settings.sites[Site::X as usize] = false;
        settings.fixupx = true;
        settings.write(&dir).unwrap();
        assert_eq!(
            fs::read_to_string(dir.join("settings")).unwrap(),
            "instagram=1\nx=0\nyoutube=1\ntiktok=1\nthreads=1\nreddit=1\nfixupx=1\n"
        );
        let back = Settings::read(&dir);
        assert_eq!(back.sites, settings.sites);
        assert!(back.fixupx);

        // Unknown keys, bad values and junk lines keep the defaults.
        fs::write(
            dir.join("settings"),
            "nope=0\nreddit=maybe\njunk\ntiktok=0\n",
        )
        .unwrap();
        let odd = Settings::read(&dir);
        assert!(odd.sites[Site::Reddit as usize] && !odd.fixupx);
        assert!(!odd.sites[Site::TikTok as usize]);

        fs::remove_dir_all(&dir).unwrap();
    }
}
