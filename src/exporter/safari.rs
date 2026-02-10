use anyhow::{anyhow, Result};
use std::path::Path;

use super::{Bookmark, HistoryEntry};

pub fn extract_bookmarks(profile_path: &Path) -> Result<Option<Vec<Bookmark>>> {
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

pub fn extract_history(profile_path: &Path) -> Result<Option<HistoryEntry>> {
    let history_path = profile_path.join("History.db");
    if !history_path.exists() {
        return Ok(None);
    }

    Ok(None)
}

fn extract_safari_bookmarks(bookmarks_path: &Path) -> Result<Option<Vec<Bookmark>>> {
    let content = std::fs::read(bookmarks_path)?;
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
