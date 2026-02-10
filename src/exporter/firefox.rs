use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::{Path, PathBuf};

use super::{Bookmark, HistoryEntry, UrlEntry};

pub fn extract_bookmarks(profile_path: &Path) -> Result<Option<Vec<Bookmark>>> {
    let places_path = if profile_path.extension().and_then(|s| s.to_str()) == Some("sqlite") {
        profile_path.to_path_buf()
    } else {
        profile_path.join("places.sqlite")
    };

    if !places_path.exists() {
        return Ok(None);
    }

    extract_firefox_bookmarks(&places_path)
}

pub fn extract_history(profile_path: &Path) -> Result<Option<HistoryEntry>> {
    let places_path = profile_path.join("places.sqlite");
    if !places_path.exists() {
        return Ok(None);
    }

    extract_firefox_history(&places_path)
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
                        DateTime::from_timestamp((ts as i64) / 1000000, 0)
                            .unwrap_or_else(Utc::now),
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
