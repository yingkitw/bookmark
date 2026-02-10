mod chrome;
mod firefox;
mod safari;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::browser::Browser;

#[derive(Debug, Serialize, Deserialize)]
pub struct BrowserData {
    pub browser: String,
    pub profile: String,
    pub export_date: DateTime<Utc>,
    pub bookmarks: Option<Vec<Bookmark>>,
    pub history: Option<HistoryEntry>,
    pub passwords: Option<Vec<Password>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bookmark {
    pub id: String,
    pub title: String,
    pub url: Option<String>,
    pub folder: Option<String>,
    pub date_added: Option<DateTime<Utc>>,
    pub children: Option<Vec<Bookmark>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub urls: Vec<UrlEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UrlEntry {
    pub url: String,
    pub title: String,
    pub visit_count: i64,
    pub last_visit: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Password {
    pub url: String,
    pub username: String,
    pub password: String,
    pub form_data: Option<HashMap<String, String>>,
}

/// Load bookmark and history data directly from browser databases (in-memory, no file I/O)
pub fn load_browser_data(
    browser_name: &str,
    data_type: &str,
) -> Result<(Vec<Bookmark>, Vec<UrlEntry>)> {
    let browsers: Vec<&str> = if browser_name == "all" {
        vec!["chrome", "firefox", "safari", "edge"]
    } else {
        vec![browser_name]
    };

    let want_bookmarks = matches!(data_type, "bookmarks" | "both");
    let want_history = matches!(data_type, "history" | "both");

    let mut all_bookmarks = Vec::new();
    let mut all_history = Vec::new();

    for name in browsers {
        let browser = match Browser::from_str(name) {
            Ok(b) => b,
            Err(_) => continue,
        };
        let profiles = match browser.find_profiles(None) {
            Ok(p) => p,
            Err(e) => {
                log::debug!("No profiles for {}: {}", name, e);
                continue;
            }
        };

        for profile_path in &profiles {
            if want_bookmarks {
                match extract_bookmarks(&browser, profile_path) {
                    Ok(Some(b)) => all_bookmarks.extend(b),
                    Ok(None) => {}
                    Err(e) => log::debug!("Failed to extract bookmarks from {}: {}", name, e),
                }
            }
            if want_history {
                match extract_history(&browser, profile_path) {
                    Ok(Some(h)) => all_history.extend(h.urls),
                    Ok(None) => {}
                    Err(e) => log::debug!("Failed to extract history from {}: {}", name, e),
                }
            }
        }
    }

    Ok((all_bookmarks, all_history))
}

pub fn export_data(
    browser_name: &str,
    data_type: &str,
    output_file: Option<PathBuf>,
    profile_dir: Option<PathBuf>,
) -> Result<()> {
    let browser = Browser::from_str(browser_name)?;
    let profiles = browser.find_profiles(profile_dir.as_deref())?;

    if profiles.is_empty() {
        return Err(anyhow!("No profiles found for {}", browser_name));
    }

    let mut all_data = Vec::new();

    for profile_path in profiles {
        let profile_name = profile_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let mut browser_data = BrowserData {
            browser: browser_name.to_string(),
            profile: profile_name,
            export_date: Utc::now(),
            bookmarks: None,
            history: None,
            passwords: None,
        };

        match data_type {
            "bookmarks" => {
                browser_data.bookmarks = extract_bookmarks(&browser, &profile_path)?;
            }
            "history" => {
                browser_data.history = extract_history(&browser, &profile_path)?;
            }
            "passwords" => {
                browser_data.passwords = extract_passwords(&browser, &profile_path)?;
            }
            "all" => {
                browser_data.bookmarks = extract_bookmarks(&browser, &profile_path)?;
                browser_data.history = extract_history(&browser, &profile_path)?;
                browser_data.passwords = extract_passwords(&browser, &profile_path)?;
            }
            _ => return Err(anyhow!("Invalid data type: {}", data_type)),
        }

        all_data.push(browser_data);
    }

    let yaml_content = serde_yaml::to_string(&all_data)?;

    match output_file {
        Some(path) => {
            fs::write(&path, yaml_content)?;
            println!("Data exported to {}", path.display());
        }
        None => {
            println!("{}", yaml_content);
        }
    }

    Ok(())
}

// --- Browser dispatch ---

fn extract_bookmarks(
    browser: &Browser,
    profile_path: &std::path::Path,
) -> Result<Option<Vec<Bookmark>>> {
    match browser {
        Browser::Chrome | Browser::Edge => chrome::extract_bookmarks(profile_path),
        Browser::Firefox => firefox::extract_bookmarks(profile_path),
        Browser::Safari => safari::extract_bookmarks(profile_path),
    }
}

fn extract_history(
    browser: &Browser,
    profile_path: &std::path::Path,
) -> Result<Option<HistoryEntry>> {
    match browser {
        Browser::Chrome | Browser::Edge => chrome::extract_history(profile_path),
        Browser::Firefox => firefox::extract_history(profile_path),
        Browser::Safari => safari::extract_history(profile_path),
    }
}

fn extract_passwords(
    browser: &Browser,
    profile_path: &std::path::Path,
) -> Result<Option<Vec<Password>>> {
    match browser {
        Browser::Chrome | Browser::Edge => {
            let login_data_path = profile_path.join("Login Data");
            if !login_data_path.exists() {
                return Ok(None);
            }
            Ok(None)
        }
        Browser::Firefox => {
            let signons_path = profile_path.join("signons.sqlite");
            if !signons_path.exists() {
                let key4_path = profile_path.join("key4.db");
                if !key4_path.exists() {
                    return Ok(None);
                }
                return Ok(None);
            }
            Ok(None)
        }
        Browser::Safari => Ok(None),
    }
}
