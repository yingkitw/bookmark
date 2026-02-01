use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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

fn extract_bookmarks(browser: &Browser, profile_path: &Path) -> Result<Option<Vec<Bookmark>>> {
    match browser {
        Browser::Chrome | Browser::Edge => {
            let bookmarks_path = profile_path.join("Bookmarks");
            if !bookmarks_path.exists() {
                return Ok(None);
            }

            let content = fs::read_to_string(bookmarks_path)?;
            let json: serde_json::Value = serde_json::from_str(&content)?;

            Ok(Some(parse_chrome_bookmarks(&json)?))
        }
        Browser::Firefox => {
            let places_path = if profile_path.extension().and_then(|s| s.to_str()) == Some("sqlite")
            {
                profile_path.to_path_buf()
            } else {
                profile_path.join("places.sqlite")
            };

            if !places_path.exists() {
                return Ok(None);
            }

            extract_firefox_bookmarks(&places_path)
        }
        Browser::Safari => {
            // First try the default location
            let bookmarks_path = profile_path.join("Bookmarks.plist");
            if bookmarks_path.exists() {
                match extract_safari_bookmarks(&bookmarks_path) {
                    Ok(bookmarks) => return Ok(bookmarks),
                    Err(e) => {
                        if e.to_string().contains("permitted") {
                            // Fall through to manual copy instructions
                        } else {
                            return Err(e);
                        }
                    }
                }
            }

            // If default location failed, check if the profile_path is already a copied file
            if profile_path.extension().and_then(|s| s.to_str()) == Some("plist") {
                return extract_safari_bookmarks(profile_path);
            }

            // Provide helpful error message with manual copy instructions
            Err(anyhow!(
                "Safari bookmarks are protected. Please manually copy the Bookmarks.plist file:\n\n\
                1. Open Finder\n\
                2. Press Shift+Command+G\n\
                3. Enter: ~/Library/Safari/\n\
                4. Copy Bookmarks.plist to your Desktop or Downloads\n\
                5. Run: cargo run -- export --browser safari --profile-dir ~/Desktop/Bookmarks.plist --data-type bookmarks --output safari-bookmarks.yaml"
            ))
        }
    }
}

fn extract_history(browser: &Browser, profile_path: &Path) -> Result<Option<HistoryEntry>> {
    match browser {
        Browser::Chrome | Browser::Edge => {
            let history_path = profile_path.join("History");
            if !history_path.exists() {
                return Ok(None);
            }

            extract_chrome_history(&history_path)
        }
        Browser::Firefox => {
            let places_path = profile_path.join("places.sqlite");
            if !places_path.exists() {
                return Ok(None);
            }

            extract_firefox_history(&places_path)
        }
        Browser::Safari => {
            let history_path = profile_path.join("History.db");
            if !history_path.exists() {
                return Ok(None);
            }

            Ok(None)
        }
    }
}

fn extract_passwords(browser: &Browser, profile_path: &Path) -> Result<Option<Vec<Password>>> {
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

fn parse_chrome_bookmarks(json: &serde_json::Value) -> Result<Vec<Bookmark>> {
    let mut bookmarks = Vec::new();

    if let Some(roots) = json.get("roots").and_then(|r| r.as_object()) {
        for (folder_name, folder_data) in roots {
            bookmarks.extend(parse_bookmark_folder(
                folder_data,
                Some(folder_name.clone()),
            )?);
        }
    }

    Ok(bookmarks)
}

fn parse_bookmark_folder(
    folder: &serde_json::Value,
    folder_name: Option<String>,
) -> Result<Vec<Bookmark>> {
    let mut bookmarks = Vec::new();

    if let Some(children) = folder.get("children").and_then(|c| c.as_array()) {
        for child in children {
            if let Some(obj) = child.as_object() {
                if obj.get("type").and_then(|t| t.as_str()) == Some("url") {
                    let bookmark = Bookmark {
                        id: obj
                            .get("id")
                            .and_then(|i| i.as_str())
                            .unwrap_or("")
                            .to_string(),
                        title: obj
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("")
                            .to_string(),
                        url: obj
                            .get("url")
                            .and_then(|u| u.as_str())
                            .map(|s| s.to_string()),
                        folder: folder_name.clone(),
                        date_added: obj
                            .get("date_added")
                            .and_then(|d| d.as_str())
                            .and_then(|s| s.parse::<i64>().ok())
                            .map(|ts| {
                                DateTime::from_timestamp((ts - 11644473600000000) / 1000000, 0)
                                    .unwrap_or_else(Utc::now)
                            }),
                        children: None,
                    };
                    bookmarks.push(bookmark);
                } else if obj.get("type").and_then(|t| t.as_str()) == Some("folder") {
                    let subfolder_name = obj
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("")
                        .to_string();
                    let full_folder_name = match folder_name {
                        Some(ref parent) => format!("{}/{}", parent, subfolder_name),
                        None => subfolder_name,
                    };
                    bookmarks.extend(parse_bookmark_folder(child, Some(full_folder_name))?);
                }
            }
        }
    }

    Ok(bookmarks)
}

fn extract_safari_bookmarks(bookmarks_path: &Path) -> Result<Option<Vec<Bookmark>>> {
    let content = fs::read(bookmarks_path)?;
    let plist: plist::Value = plist::from_bytes(&content)?;

    let mut bookmarks = Vec::new();

    if let Some(dict) = plist.into_dictionary() {
        if let Some(children) = dict.get("Children").and_then(|c| c.as_array()) {
            for item in children {
                if let Some(bookmark_dict) = item.as_dictionary() {
                    if bookmark_dict
                        .get("WebBookmarkType")
                        .and_then(|t| t.as_string())
                        == Some("WebBookmarkTypeLeaf")
                    {
                        let title = bookmark_dict
                            .get("URIDictionary")
                            .and_then(|d| d.as_dictionary())
                            .and_then(|d| d.get("title"))
                            .and_then(|t| t.as_string())
                            .unwrap_or("")
                            .to_string();

                        let bookmark = Bookmark {
                            id: title.clone(),
                            title: title.clone(),
                            url: bookmark_dict
                                .get("URLString")
                                .and_then(|u| u.as_string())
                                .map(|s| s.to_string()),
                            folder: None,
                            date_added: None,
                            children: None,
                        };
                        bookmarks.push(bookmark);
                    }
                }
            }
        }
    }

    Ok(Some(bookmarks))
}

fn extract_chrome_history(history_path: &Path) -> Result<Option<HistoryEntry>> {
    let conn = rusqlite::Connection::open(history_path)?;

    let mut stmt = conn.prepare(
        "SELECT url, title, visit_count, last_visit_time 
         FROM urls 
         ORDER BY last_visit_time DESC 
         LIMIT 10000",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(UrlEntry {
            url: row.get(0)?,
            title: row.get(1)?,
            visit_count: row.get(2)?,
            last_visit: row.get::<_, Option<i64>>(3)?.map(|ts| {
                DateTime::from_timestamp((ts - 11644473600000000) / 1000000, 0)
                    .unwrap_or_else(Utc::now)
            }),
        })
    })?;

    let mut urls = Vec::new();
    for row in rows {
        urls.push(row?);
    }

    Ok(Some(HistoryEntry { urls }))
}

fn extract_firefox_bookmarks(places_path: &Path) -> Result<Option<Vec<Bookmark>>> {
    // Try to copy the database first to avoid lock issues
    let temp_path = PathBuf::from("/tmp/places_copy.sqlite");

    if let Err(e) = fs::copy(places_path, &temp_path) {
        if e.to_string().contains("permission") || e.to_string().contains("locked") {
            return Err(anyhow!(
                "Firefox is running. Please close Firefox and try again. {}",
                e
            ));
        }
        return Err(e.into());
    }

    let conn = rusqlite::Connection::open_with_flags(
        &temp_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?;

    let mut stmt = conn.prepare(
        "SELECT b.id, b.title, p.url, b.dateAdded, p2.title as folder_title
         FROM moz_bookmarks b
         LEFT JOIN moz_places p ON b.fk = p.id
         LEFT JOIN moz_bookmarks p2 ON b.parent = p2.id
         WHERE b.type = 1 AND p.url IS NOT NULL
         ORDER BY b.dateAdded DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(Bookmark {
            id: row.get::<_, i64>(0)?.to_string(),
            title: row
                .get::<_, Option<String>>(1)?
                .unwrap_or_else(|| "".to_string()),
            url: row.get(2)?,
            folder: match row.get::<_, Option<String>>(3) {
                Ok(folder) => folder,
                Err(_) => None,
            },
            date_added: match row.get::<_, i64>(4) {
                Ok(ts) => Some(DateTime::from_timestamp(ts / 1000000, 0).unwrap_or_else(Utc::now)),
                Err(_) => match row.get::<_, f64>(4) {
                    Ok(ts) => Some(
                        DateTime::from_timestamp((ts as i64) / 1000000, 0).unwrap_or_else(Utc::now),
                    ),
                    Err(_) => None,
                },
            },
            children: None,
        })
    })?;

    let mut bookmarks = Vec::new();
    for row in rows {
        bookmarks.push(row?);
    }

    Ok(Some(bookmarks))
}

fn extract_firefox_history(places_path: &Path) -> Result<Option<HistoryEntry>> {
    // Try to copy the database first to avoid lock issues
    let temp_path = PathBuf::from("/tmp/places_copy_history.sqlite");

    if let Err(e) = fs::copy(places_path, &temp_path) {
        if e.to_string().contains("permission") || e.to_string().contains("locked") {
            return Err(anyhow!(
                "Firefox is running. Please close Firefox and try again. {}",
                e
            ));
        }
        return Err(e.into());
    }

    let conn = rusqlite::Connection::open_with_flags(
        &temp_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?;

    let mut stmt = conn.prepare(
        "SELECT p.url, p.title, p.visit_count, p.last_visit_date 
         FROM moz_places p
         WHERE p.url IS NOT NULL
         ORDER BY p.last_visit_date DESC 
         LIMIT 10000",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(UrlEntry {
            url: row.get(0)?,
            title: row
                .get::<_, Option<String>>(1)?
                .unwrap_or_else(|| "".to_string()),
            visit_count: row.get::<_, Option<i64>>(2)?.unwrap_or(0),
            last_visit: row
                .get::<_, Option<i64>>(3)?
                .map(|ts| DateTime::from_timestamp(ts / 1000000, 0).unwrap_or_else(Utc::now)),
        })
    })?;

    let mut urls = Vec::new();
    for row in rows {
        urls.push(row?);
    }

    Ok(Some(HistoryEntry { urls }))
}
