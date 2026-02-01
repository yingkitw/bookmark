use anyhow::Result;
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use url::Url;

use crate::exporter::Bookmark;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeduplicationConfig {
    pub normalize_urls: bool,
    pub ignore_query_params: bool,
    pub ignore_fragment: bool,
    pub ignore_www: bool,
    pub ignore_protocol: bool,
    pub case_sensitive: bool,
    pub merge_strategy: MergeStrategy,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum MergeStrategy {
    KeepFirst,
    KeepLast,
    KeepMostRecent,
    KeepMostFrequent,
    MergeMetadata,
}

impl Default for DeduplicationConfig {
    fn default() -> Self {
        Self {
            normalize_urls: true,
            ignore_query_params: true,
            ignore_fragment: true,
            ignore_www: true,
            ignore_protocol: true,
            case_sensitive: false,
            merge_strategy: MergeStrategy::MergeMetadata,
        }
    }
}

#[derive(Debug)]
pub struct DeduplicationResult {
    pub unique_bookmarks: Vec<Bookmark>,
    pub duplicates_removed: usize,
    pub duplicates_found: usize,
    pub merge_summary: HashMap<String, usize>,
}

pub struct BookmarkDeduplicator {
    config: DeduplicationConfig,
}

impl BookmarkDeduplicator {
    pub fn new(config: DeduplicationConfig) -> Self {
        Self { config }
    }

    pub fn deduplicate(&self, bookmarks: &[Bookmark]) -> Result<DeduplicationResult> {
        let mut url_groups: HashMap<String, Vec<Bookmark>> = HashMap::new();
        let mut seen_urls: HashSet<String> = HashSet::new();

        // Group bookmarks by normalized URL
        for bookmark in bookmarks {
            if let Some(ref url) = bookmark.url {
                let normalized_url = self.normalize_url(url)?;

                if seen_urls.contains(&normalized_url) {
                    if let Some(group) = url_groups.get_mut(&normalized_url) {
                        group.push(bookmark.clone());
                    }
                } else {
                    seen_urls.insert(normalized_url.clone());
                    url_groups.insert(normalized_url, vec![bookmark.clone()]);
                }
            }
        }

        let mut unique_bookmarks = Vec::new();
        let mut duplicates_removed = 0;
        let mut duplicates_found = 0;
        let mut merge_summary = HashMap::new();

        for (normalized_url, group) in url_groups {
            if group.len() == 1 {
                unique_bookmarks.push(group.into_iter().next().unwrap());
            } else {
                duplicates_found += group.len() - 1;

                let merged = self.merge_bookmarks(&group)?;
                duplicates_removed += group.len() - 1;

                merge_summary.insert(normalized_url, group.len());

                unique_bookmarks.push(merged);
            }
        }

        Ok(DeduplicationResult {
            unique_bookmarks,
            duplicates_removed,
            duplicates_found,
            merge_summary,
        })
    }

    fn normalize_url(&self, url_str: &str) -> Result<String> {
        let mut url = Url::parse(url_str)?;

        if self.config.ignore_protocol {
            url.set_scheme("http").ok(); // Set to http for consistency
        }

        if self.config.ignore_www {
            let host = url.host_str().unwrap_or("").to_string();
            if host.starts_with("www.") {
                let new_host = &host[4..];
                url.set_host(Some(new_host))?;
            }
        }

        if self.config.normalize_urls {
            // Remove trailing slashes
            let path = url.path().trim_end_matches('/').to_string();
            url.set_path(&path);
        }

        if self.config.ignore_query_params {
            url.set_query(None);
        }

        if self.config.ignore_fragment {
            url.set_fragment(None);
        }

        let mut normalized = url.to_string();

        if !self.config.case_sensitive {
            normalized = normalized.to_lowercase();
        }

        Ok(normalized)
    }

    fn merge_bookmarks(&self, bookmarks: &[Bookmark]) -> Result<Bookmark> {
        match self.config.merge_strategy {
            MergeStrategy::KeepFirst => Ok(bookmarks[0].clone()),
            MergeStrategy::KeepLast => Ok(bookmarks[bookmarks.len() - 1].clone()),
            MergeStrategy::KeepMostRecent => {
                let most_recent = bookmarks
                    .iter()
                    .max_by_key(|b| b.date_added.unwrap_or_else(Utc::now))
                    .unwrap();
                Ok(most_recent.clone())
            }
            MergeStrategy::KeepMostFrequent => {
                // Count frequency of titles
                let mut title_counts: HashMap<String, usize> = HashMap::new();
                for bookmark in bookmarks {
                    *title_counts.entry(bookmark.title.clone()).or_insert(0) += 1;
                }

                let most_frequent_title = title_counts
                    .iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(title, _)| title.clone())
                    .unwrap_or_else(|| bookmarks[0].title.clone());

                // Find first bookmark with the most frequent title
                let bookmark = bookmarks
                    .iter()
                    .find(|b| b.title == most_frequent_title)
                    .unwrap_or(&bookmarks[0]);

                Ok(bookmark.clone())
            }
            MergeStrategy::MergeMetadata => {
                // Create a merged bookmark with the best metadata
                let first_bookmark = &bookmarks[0];

                // Use the title from the most recent bookmark with a non-empty title
                let title = bookmarks
                    .iter()
                    .filter(|b| !b.title.is_empty())
                    .max_by_key(|b| b.date_added.unwrap_or_else(Utc::now))
                    .map(|b| b.title.clone())
                    .unwrap_or_else(|| first_bookmark.title.clone());

                // Use the most recent date
                let date_added = bookmarks.iter().filter_map(|b| b.date_added).max();

                // Combine folders from all sources
                let mut folders = Vec::new();
                for bookmark in bookmarks {
                    if let Some(ref folder) = bookmark.folder {
                        if !folders.contains(&folder.clone()) {
                            folders.push(folder.clone());
                        }
                    }
                }

                let folder = if folders.is_empty() {
                    None
                } else if folders.len() == 1 {
                    Some(folders[0].clone())
                } else {
                    Some(format!("Merged: {}", folders.join(", ")))
                };

                Ok(Bookmark {
                    id: first_bookmark.id.clone(),
                    title,
                    url: first_bookmark.url.clone(),
                    folder,
                    date_added,
                    children: None, // Don't merge children for simplicity
                })
            }
        }
    }
}

pub fn find_potential_duplicates(bookmarks: &[Bookmark]) -> Result<Vec<(Bookmark, Bookmark, f64)>> {
    let mut duplicates = Vec::new();

    for (i, bookmark1) in bookmarks.iter().enumerate() {
        for bookmark2 in bookmarks.iter().skip(i + 1) {
            if let (Some(url1), Some(url2)) = (&bookmark1.url, &bookmark2.url) {
                let similarity = calculate_url_similarity(url1, url2)?;

                if similarity > 0.8 {
                    duplicates.push((bookmark1.clone(), bookmark2.clone(), similarity));
                }
            }
        }
    }

    Ok(duplicates)
}

fn calculate_url_similarity(url1: &str, url2: &str) -> Result<f64> {
    let parsed1 = Url::parse(url1)?;
    let parsed2 = Url::parse(url2)?;

    let mut score = 0.0;
    let mut factors = 0;

    // Compare domains
    if parsed1.host_str() == parsed2.host_str() {
        score += 0.5;
    }
    factors += 1;

    // Compare paths
    let path1 = normalize_path(parsed1.path());
    let path2 = normalize_path(parsed2.path());

    if path1 == path2 {
        score += 0.3;
    } else {
        // Partial path match
        let path_parts1: HashSet<_> = path1.split('/').collect();
        let path_parts2: HashSet<_> = path2.split('/').collect();
        let path_similarity = jaccard_similarity(&path_parts1, &path_parts2);
        score += path_similarity * 0.3;
    }
    factors += 1;

    // Compare query parameters
    if let (Some(query1), Some(query2)) = (parsed1.query(), parsed2.query()) {
        let params1: HashSet<&str> = query1.split('&').collect();
        let params2: HashSet<&str> = query2.split('&').collect();

        let query_similarity = jaccard_similarity(&params1, &params2);
        score += query_similarity * 0.2;
    } else if parsed1.query().is_none() && parsed2.query().is_none() {
        score += 0.2;
    }
    factors += 1;

    Ok(score / factors as f64)
}

fn normalize_path(path: &str) -> String {
    path.trim_end_matches('/').to_lowercase()
}

fn jaccard_similarity<T: std::hash::Hash + Eq>(set1: &HashSet<T>, set2: &HashSet<T>) -> f64 {
    if set1.is_empty() && set2.is_empty() {
        return 1.0;
    }

    let intersection: HashSet<_> = set1.intersection(set2).collect();
    let union: HashSet<_> = set1.union(set2).collect();

    intersection.len() as f64 / union.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_normalization() {
        let config = DeduplicationConfig::default();
        let deduplicator = BookmarkDeduplicator::new(config);

        assert_eq!(
            deduplicator
                .normalize_url("https://www.example.com/path?param=value#section")
                .unwrap(),
            "http://example.com/path"
        );

        assert_eq!(
            deduplicator
                .normalize_url("https://example.com/path")
                .unwrap(),
            "http://example.com/path"
        );
    }

    #[test]
    fn test_deduplication() {
        let config = DeduplicationConfig::default();
        let deduplicator = BookmarkDeduplicator::new(config);

        let bookmarks = vec![
            Bookmark {
                id: "1".to_string(),
                title: "Example".to_string(),
                url: Some("https://www.example.com".to_string()),
                folder: Some("folder1".to_string()),
                date_added: None,
                children: None,
            },
            Bookmark {
                id: "2".to_string(),
                title: "Example Site".to_string(),
                url: Some("http://example.com".to_string()),
                folder: Some("folder2".to_string()),
                date_added: None,
                children: None,
            },
        ];

        let result = deduplicator.deduplicate(&bookmarks).unwrap();
        assert_eq!(result.unique_bookmarks.len(), 1);
        assert_eq!(result.duplicates_removed, 1);
    }

    #[test]
    fn test_trailing_slash_normalization() {
        let config = DeduplicationConfig::default();
        let deduplicator = BookmarkDeduplicator::new(config);

        assert_eq!(
            deduplicator.normalize_url("https://example.com/path/").unwrap(),
            "http://example.com/path"
        );

        assert_eq!(
            deduplicator.normalize_url("https://example.com/path").unwrap(),
            "http://example.com/path"
        );
    }

    #[test]
    fn test_query_params_removal() {
        let config = DeduplicationConfig::default();
        let deduplicator = BookmarkDeduplicator::new(config);

        assert_eq!(
            deduplicator
                .normalize_url("https://example.com/path?utm_source=google&id=123")
                .unwrap(),
            "http://example.com/path"
        );

        assert_eq!(
            deduplicator
                .normalize_url("https://example.com/path#section")
                .unwrap(),
            "http://example.com/path"
        );
    }

    #[test]
    fn test_case_insensitive() {
        let config = DeduplicationConfig::default();
        let deduplicator = BookmarkDeduplicator::new(config);

        let url1 = deduplicator.normalize_url("https://EXAMPLE.com/PATH").unwrap();
        let url2 = deduplicator.normalize_url("https://example.com/path").unwrap();

        assert_eq!(url1, url2);
    }

    #[test]
    fn test_multiple_duplicates() {
        let config = DeduplicationConfig::default();
        let deduplicator = BookmarkDeduplicator::new(config);

        let bookmarks = vec![
            Bookmark {
                id: "1".to_string(),
                title: "Example 1".to_string(),
                url: Some("https://www.example.com".to_string()),
                folder: None,
                date_added: None,
                children: None,
            },
            Bookmark {
                id: "2".to_string(),
                title: "Example 2".to_string(),
                url: Some("http://example.com".to_string()),
                folder: None,
                date_added: None,
                children: None,
            },
            Bookmark {
                id: "3".to_string(),
                title: "Example 3".to_string(),
                url: Some("https://example.com/".to_string()),
                folder: None,
                date_added: None,
                children: None,
            },
        ];

        let result = deduplicator.deduplicate(&bookmarks).unwrap();
        assert_eq!(result.unique_bookmarks.len(), 1);
        assert_eq!(result.duplicates_removed, 2);
        assert_eq!(result.duplicates_found, 2);
    }

    #[test]
    fn test_no_duplicates() {
        let config = DeduplicationConfig::default();
        let deduplicator = BookmarkDeduplicator::new(config);

        let bookmarks = vec![
            Bookmark {
                id: "1".to_string(),
                title: "GitHub".to_string(),
                url: Some("https://github.com".to_string()),
                folder: None,
                date_added: None,
                children: None,
            },
            Bookmark {
                id: "2".to_string(),
                title: "Rust".to_string(),
                url: Some("https://rust-lang.org".to_string()),
                folder: None,
                date_added: None,
                children: None,
            },
        ];

        let result = deduplicator.deduplicate(&bookmarks).unwrap();
        assert_eq!(result.unique_bookmarks.len(), 2);
        assert_eq!(result.duplicates_removed, 0);
        assert_eq!(result.duplicates_found, 0);
    }

    #[test]
    fn test_bookmark_without_url() {
        let config = DeduplicationConfig::default();
        let deduplicator = BookmarkDeduplicator::new(config);

        let bookmarks = vec![
            Bookmark {
                id: "1".to_string(),
                title: "Example".to_string(),
                url: Some("https://example.com".to_string()),
                folder: None,
                date_added: None,
                children: None,
            },
            Bookmark {
                id: "2".to_string(),
                title: "No URL".to_string(),
                url: None,
                folder: None,
                date_added: None,
                children: None,
            },
        ];

        let result = deduplicator.deduplicate(&bookmarks).unwrap();
        // Bookmarks without URLs are currently dropped by deduplication
        assert_eq!(result.unique_bookmarks.len(), 1);
    }

    #[test]
    fn test_merge_strategies() {
        use chrono::Utc;

        let bookmarks = vec![
            Bookmark {
                id: "1".to_string(),
                title: "First".to_string(),
                url: Some("https://example.com".to_string()),
                folder: Some("folder1".to_string()),
                date_added: None,
                children: None,
            },
            Bookmark {
                id: "2".to_string(),
                title: "Last".to_string(),
                url: Some("http://example.com".to_string()),
                folder: Some("folder2".to_string()),
                date_added: Some(Utc::now()),
                children: None,
            },
        ];

        // Test KeepFirst
        let config = DeduplicationConfig {
            merge_strategy: MergeStrategy::KeepFirst,
            ..Default::default()
        };
        let deduplicator = BookmarkDeduplicator::new(config);
        let result = deduplicator.deduplicate(&bookmarks).unwrap();
        assert_eq!(result.unique_bookmarks[0].title, "First");
        assert_eq!(result.unique_bookmarks[0].id, "1");

        // Test KeepLast
        let config = DeduplicationConfig {
            merge_strategy: MergeStrategy::KeepLast,
            ..Default::default()
        };
        let deduplicator = BookmarkDeduplicator::new(config);
        let result = deduplicator.deduplicate(&bookmarks).unwrap();
        assert_eq!(result.unique_bookmarks[0].title, "Last");
        assert_eq!(result.unique_bookmarks[0].id, "2");
    }

    #[test]
    fn test_complex_url_normalization() {
        let config = DeduplicationConfig::default();
        let deduplicator = BookmarkDeduplicator::new(config);

        // Test with various URL patterns
        let test_cases = vec![
            ("https://www.example.com/path", "http://example.com/path"),
            ("https://example.com/path/", "http://example.com/path"),
            ("https://example.com/path?foo=bar", "http://example.com/path"),
            ("https://example.com/path#section", "http://example.com/path"),
            ("https://EXAMPLE.COM/path", "http://example.com/path"),
            ("http://www.example.com/path/", "http://example.com/path"),
        ];

        for (input, expected) in test_cases {
            let result = deduplicator.normalize_url(input).unwrap();
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }
}
