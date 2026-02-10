use anyhow::Result;
use chrono::{DateTime, Utc};
use std::path::Path;

use super::{Bookmark, HistoryEntry, UrlEntry};

pub fn extract_bookmarks(profile_path: &Path) -> Result<Option<Vec<Bookmark>>> {
    let bookmarks_path = profile_path.join("Bookmarks");
    if !bookmarks_path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(bookmarks_path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    Ok(Some(parse_chrome_bookmarks(&json)?))
}

pub fn extract_history(profile_path: &Path) -> Result<Option<HistoryEntry>> {
    let history_path = profile_path.join("History");
    if !history_path.exists() {
        return Ok(None);
    }

    let conn = rusqlite::Connection::open(&history_path)?;

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
