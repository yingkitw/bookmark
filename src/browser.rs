use anyhow::{anyhow, Result};
use dirs;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum Browser {
    Chrome,
    Firefox,
    Safari,
    Edge,
}

impl FromStr for Browser {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "chrome" => Ok(Browser::Chrome),
            "firefox" => Ok(Browser::Firefox),
            "safari" => Ok(Browser::Safari),
            "edge" => Ok(Browser::Edge),
            _ => Err(anyhow!("Unsupported browser: {}", s)),
        }
    }
}

impl Browser {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "chrome" => Ok(Browser::Chrome),
            "firefox" => Ok(Browser::Firefox),
            "safari" => Ok(Browser::Safari),
            "edge" => Ok(Browser::Edge),
            _ => Err(anyhow!("Unsupported browser: {}", s)),
        }
    }

    pub fn get_default_data_dir(&self) -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;

        match self {
            Browser::Chrome => {
                if cfg!(target_os = "macos") {
                    Ok(home.join("Library/Application Support/Google/Chrome"))
                } else if cfg!(target_os = "windows") {
                    let app_data = dirs::data_dir()
                        .ok_or_else(|| anyhow!("Could not find AppData directory"))?;
                    Ok(app_data.join("Google/Chrome/User Data"))
                } else {
                    Ok(home.join(".config/google-chrome"))
                }
            }
            Browser::Firefox => {
                if cfg!(target_os = "macos") {
                    Ok(home.join("Library/Application Support/Firefox/Profiles"))
                } else if cfg!(target_os = "windows") {
                    let app_data = dirs::data_dir()
                        .ok_or_else(|| anyhow!("Could not find AppData directory"))?;
                    Ok(app_data.join("Mozilla/Firefox/Profiles"))
                } else {
                    Ok(home.join(".mozilla/firefox"))
                }
            }
            Browser::Safari => {
                if cfg!(target_os = "macos") {
                    Ok(home.join("Library/Safari"))
                } else {
                    Err(anyhow!("Safari is only available on macOS"))
                }
            }
            Browser::Edge => {
                if cfg!(target_os = "macos") {
                    Ok(home.join("Library/Application Support/Microsoft Edge"))
                } else if cfg!(target_os = "windows") {
                    let app_data = dirs::data_dir()
                        .ok_or_else(|| anyhow!("Could not find AppData directory"))?;
                    Ok(app_data.join("Microsoft/Edge/User Data"))
                } else {
                    Ok(home.join(".config/microsoft-edge"))
                }
            }
        }
    }

    pub fn find_profiles(&self, custom_dir: Option<&Path>) -> Result<Vec<PathBuf>> {
        let base_dir = match custom_dir {
            Some(dir) => dir.to_path_buf(),
            None => self.get_default_data_dir()?,
        };

        let mut profiles = Vec::new();

        match self {
            Browser::Chrome | Browser::Edge => {
                for entry in fs::read_dir(&base_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        let profile_name = path.file_name().unwrap().to_string_lossy();
                        if profile_name.contains("Profile") || profile_name == "Default" {
                            if path.join("Bookmarks").exists() {
                                profiles.push(path);
                            }
                        }
                    }
                }

                if profiles.is_empty() {
                    let default_profile = base_dir.join("Default");
                    if default_profile.join("Bookmarks").exists() {
                        profiles.push(default_profile);
                    }
                }
            }
            Browser::Firefox => {
                for entry in fs::read_dir(&base_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        if path.join("places.sqlite").exists() {
                            profiles.push(path);
                        }
                    }
                }
            }
            Browser::Safari => {
                if base_dir.join("Bookmarks.plist").exists() {
                    profiles.push(base_dir);
                } else if base_dir.extension().and_then(|s| s.to_str()) == Some("plist") {
                    // If custom_dir is a plist file, use it directly
                    profiles.push(base_dir.parent().unwrap_or(&base_dir).to_path_buf());
                }
            }
        }

        Ok(profiles)
    }

    pub fn find_profiles_with_lock_check(&self, custom_dir: Option<&Path>) -> Result<Vec<PathBuf>> {
        let base_dir = match custom_dir {
            Some(dir) => dir.to_path_buf(),
            None => self.get_default_data_dir()?,
        };

        let mut profiles = Vec::new();

        match self {
            Browser::Chrome | Browser::Edge => {
                for entry in fs::read_dir(&base_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        let profile_name = path.file_name().unwrap().to_string_lossy();
                        if profile_name.contains("Profile") || profile_name == "Default" {
                            if path.join("Bookmarks").exists() {
                                profiles.push(path);
                            }
                        }
                    }
                }

                if profiles.is_empty() {
                    let default_profile = base_dir.join("Default");
                    if default_profile.join("Bookmarks").exists() {
                        profiles.push(default_profile);
                    }
                }
            }
            Browser::Firefox => {
                for entry in fs::read_dir(base_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        if path.join("places.sqlite").exists() {
                            profiles.push(path);
                        }
                    }
                }
            }
            Browser::Safari => {
                if base_dir.join("Bookmarks.plist").exists() {
                    profiles.push(base_dir);
                }
            }
        }

        Ok(profiles)
    }
}

pub fn list_all_browsers() -> Result<()> {
    let browsers = ["Chrome", "Firefox", "Safari", "Edge"];

    println!("Available browsers:");
    for browser_name in browsers.iter() {
        if let Ok(browser) = Browser::from_str(browser_name) {
            if let Ok(profiles) = browser.find_profiles(None) {
                if !profiles.is_empty() {
                    println!("  {} ({} profiles)", browser_name, profiles.len());
                } else {
                    println!("  {} (no profiles found)", browser_name);
                }
            } else {
                println!("  {} (not available)", browser_name);
            }
        }
    }

    Ok(())
}

pub fn list_profiles(browser_name: &str) -> Result<()> {
    let browser = Browser::from_str(browser_name)?;
    let profiles = browser.find_profiles(None)?;

    if profiles.is_empty() {
        println!("No profiles found for {}", browser_name);
    } else {
        println!("Profiles for {}:", browser_name);
        for (i, profile) in profiles.iter().enumerate() {
            println!("  {}: {}", i + 1, profile.display());
        }
    }

    Ok(())
}
