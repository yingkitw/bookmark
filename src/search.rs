use crate::browser::Browser;
use crate::exporter::{export_data, BrowserData};
use anyhow::{anyhow, Result};
use dialoguer::Select;
use serde_yaml;
use std::fs;
use std::path::PathBuf;

pub fn search_bookmarks(query: &str, title_only: bool, url_only: bool, limit: usize) -> Result<()> {
    // First, import all bookmarks to a temporary file
    let temp_file = PathBuf::from("/tmp/bookmark_search_data.yaml");

    let browsers = ["Chrome", "Firefox", "Safari", "Edge"];
    let mut all_bookmarks = Vec::new();

    println!("Loading bookmarks from all browsers...");

    for browser_name in browsers.iter() {
        match Browser::from_str(browser_name) {
            Ok(browser) => {
                if let Ok(profiles) = browser.find_profiles(None) {
                    if !profiles.is_empty() {
                        match export_data(browser_name, "bookmarks", Some(temp_file.clone()), None)
                        {
                            Ok(_) => {
                                // Read the exported data and extract bookmarks
                                if let Ok(content) = fs::read_to_string(&temp_file) {
                                    if let Ok(data) =
                                        serde_yaml::from_str::<Vec<BrowserData>>(&content)
                                    {
                                        for browser_data in data {
                                            if let Some(bookmarks) = browser_data.bookmarks {
                                                for bookmark in bookmarks {
                                                    if let Some(url) = &bookmark.url {
                                                        if !url.is_empty() {
                                                            all_bookmarks.push((
                                                                bookmark,
                                                                browser_name.to_string(),
                                                            ));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                // Continue with other browsers if one fails
                                continue;
                            }
                        }
                    }
                }
            }
            Err(_) => continue,
        }
    }

    if all_bookmarks.is_empty() {
        println!("No bookmarks found.");
        return Ok(());
    }

    // Filter bookmarks based on search criteria
    let filtered_bookmarks: Vec<_> = all_bookmarks
        .into_iter()
        .filter(|(bookmark, _)| {
            let query_lower = query.to_lowercase();
            let title_match = bookmark.title.to_lowercase().contains(&query_lower);
            let url_match = bookmark
                .url
                .as_ref()
                .map(|u| u.to_lowercase().contains(&query_lower))
                .unwrap_or(false);

            if title_only {
                title_match
            } else if url_only {
                url_match
            } else {
                title_match || url_match
            }
        })
        .take(limit)
        .collect();

    if filtered_bookmarks.is_empty() {
        println!("No bookmarks found matching '{}'.", query);
        return Ok(());
    }

    // Display results
    println!(
        "Found {} bookmarks matching '{}':",
        filtered_bookmarks.len(),
        query
    );
    println!();

    for (i, (bookmark, browser)) in filtered_bookmarks.iter().enumerate() {
        println!("{}. [{}] {}", i + 1, browser, bookmark.title);
        if let Some(url) = &bookmark.url {
            println!("   {}", url);
        }
        if let Some(folder) = &bookmark.folder {
            println!("   Folder: {}", folder);
        }
        println!();
    }

    // Clean up temporary file
    let _ = fs::remove_file(&temp_file);

    Ok(())
}

pub fn open_bookmark(query: &str, first: bool) -> Result<()> {
    // First, import all bookmarks to a temporary file
    let temp_file = PathBuf::from("/tmp/bookmark_open_data.yaml");

    let browsers = ["Chrome", "Firefox", "Safari", "Edge"];
    let mut all_bookmarks = Vec::new();

    println!("Searching for bookmarks to open...");

    for browser_name in browsers.iter() {
        match Browser::from_str(browser_name) {
            Ok(browser) => {
                if let Ok(profiles) = browser.find_profiles(None) {
                    if !profiles.is_empty() {
                        match export_data(browser_name, "bookmarks", Some(temp_file.clone()), None)
                        {
                            Ok(_) => {
                                // Read the exported data and extract bookmarks
                                if let Ok(content) = fs::read_to_string(&temp_file) {
                                    if let Ok(data) =
                                        serde_yaml::from_str::<Vec<BrowserData>>(&content)
                                    {
                                        for browser_data in data {
                                            if let Some(bookmarks) = browser_data.bookmarks {
                                                for bookmark in bookmarks {
                                                    if let Some(url) = &bookmark.url {
                                                        if !url.is_empty() {
                                                            all_bookmarks.push((
                                                                bookmark,
                                                                browser_name.to_string(),
                                                            ));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                // Continue with other browsers if one fails
                                continue;
                            }
                        }
                    }
                }
            }
            Err(_) => continue,
        }
    }

    if all_bookmarks.is_empty() {
        println!("No bookmarks found.");
        return Ok(());
    }

    // Filter bookmarks based on search query
    let query_lower = query.to_lowercase();
    let filtered_bookmarks: Vec<_> = all_bookmarks
        .into_iter()
        .filter(|(bookmark, _)| {
            let title_match = bookmark.title.to_lowercase().contains(&query_lower);
            let url_match = bookmark
                .url
                .as_ref()
                .map(|u| u.to_lowercase().contains(&query_lower))
                .unwrap_or(false);
            title_match || url_match
        })
        .collect();

    if filtered_bookmarks.is_empty() {
        println!("No bookmarks found matching '{}'.", query);
        return Ok(());
    }

    let bookmark_to_open = if filtered_bookmarks.len() == 1 || first {
        &filtered_bookmarks[0]
    } else {
        // Create selection list
        let items: Vec<String> = filtered_bookmarks
            .iter()
            .map(|(bookmark, browser)| format!("[{}] {}", browser, bookmark.title))
            .collect();

        let selection = Select::new()
            .with_prompt("Select bookmark to open:")
            .items(&items)
            .interact()?;

        &filtered_bookmarks[selection]
    };

    // Open the bookmark URL
    if let Some(url) = &bookmark_to_open.0.url {
        println!("Opening: {}", url);

        // Use the `open` crate to open the URL in the default browser
        match open::that(url) {
            Ok(_) => println!("Bookmark opened successfully!"),
            Err(e) => return Err(anyhow!("Failed to open bookmark: {}", e)),
        }
    } else {
        return Err(anyhow!("Selected bookmark has no URL"));
    }

    // Clean up temporary file
    let _ = fs::remove_file(&temp_file);

    Ok(())
}
