use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use url::Url;

use crate::exporter::Bookmark;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrganizationConfig {
    pub organize_by_domain: bool,
    pub organize_by_category: bool,
    pub organize_by_date: bool,
    pub custom_rules: Vec<OrganizationRule>,
    pub folder_separator: String,
    pub preserve_existing: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrganizationRule {
    pub name: String,
    pub pattern: String,
    pub folder: String,
    pub priority: i32,
}

impl Default for OrganizationConfig {
    fn default() -> Self {
        let mut custom_rules = Vec::new();

        // Social media
        custom_rules.push(OrganizationRule {
            name: "Social Media".to_string(),
            pattern: r"(facebook|twitter|x|instagram|linkedin|reddit|youtube|tiktok|snapchat)\.com"
                .to_string(),
            folder: "Social".to_string(),
            priority: 10,
        });

        // Development
        custom_rules.push(OrganizationRule {
            name: "Development".to_string(),
            pattern: r"(github|gitlab|bitbucket|stackoverflow|dev\.to|medium\.com)".to_string(),
            folder: "Development".to_string(),
            priority: 9,
        });

        // Shopping
        custom_rules.push(OrganizationRule {
            name: "Shopping".to_string(),
            pattern: r"(amazon|ebay|etsy|shopify|aliexpress|walmart|target)".to_string(),
            folder: "Shopping".to_string(),
            priority: 8,
        });

        // News
        custom_rules.push(OrganizationRule {
            name: "News".to_string(),
            pattern:
                r"(cnn|bbc|reuters|wikipedia|nytimes|washingtonpost|news\.|\.co\.|\.org\.|\.edu\.)"
                    .to_string(),
            folder: "News & Reference".to_string(),
            priority: 7,
        });

        // Entertainment
        custom_rules.push(OrganizationRule {
            name: "Entertainment".to_string(),
            pattern: r"(netflix|hulu|disney\+|spotify|apple\.music|twitch)".to_string(),
            folder: "Entertainment".to_string(),
            priority: 6,
        });

        // Work/Productivity
        custom_rules.push(OrganizationRule {
            name: "Work".to_string(),
            pattern: r"(office\.com|google\.com/docs|slack|teams|zoom|notion|trello|asana)"
                .to_string(),
            folder: "Work".to_string(),
            priority: 5,
        });

        Self {
            organize_by_domain: true,
            organize_by_category: true,
            organize_by_date: false,
            custom_rules,
            folder_separator: "/".to_string(),
            preserve_existing: true,
        }
    }
}

pub struct BookmarkOrganizer {
    config: OrganizationConfig,
}

impl BookmarkOrganizer {
    pub fn new(config: OrganizationConfig) -> Self {
        Self { config }
    }

    pub fn organize(&self, bookmarks: Vec<Bookmark>) -> Result<Vec<Bookmark>> {
        let mut organized_bookmarks = Vec::new();

        for mut bookmark in bookmarks {
            let new_folder = self.determine_folder(&bookmark);

            if self.config.preserve_existing && bookmark.folder.is_some() {
                if let Some(ref existing_folder) = bookmark.folder {
                    bookmark.folder = Some(format!(
                        "{}{}{}",
                        new_folder, self.config.folder_separator, existing_folder
                    ));
                }
            } else {
                bookmark.folder = Some(new_folder);
            }

            organized_bookmarks.push(bookmark);
        }

        Ok(organized_bookmarks)
    }

    fn determine_folder(&self, bookmark: &Bookmark) -> String {
        // Check custom rules first (sorted by priority)
        let mut sorted_rules = self.config.custom_rules.clone();
        sorted_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        if let Some(ref url_str) = bookmark.url {
            for rule in &sorted_rules {
                if let Ok(regex) = Regex::new(&rule.pattern) {
                    if regex.is_match(url_str) {
                        return rule.folder.clone();
                    }
                }
            }

            // If no custom rule matches, check domain-based organization
            if self.config.organize_by_domain {
                if let Ok(url) = Url::parse(url_str) {
                    if let Some(host) = url.host_str() {
                        return self.extract_domain_folder(host);
                    }
                }
            }

            // Category-based organization as fallback
            if self.config.organize_by_category {
                return self.categorize_by_content(url_str, &bookmark.title);
            }

            // Date-based organization as last resort
            if self.config.organize_by_date {
                return self.categorize_by_date(&bookmark.date_added);
            }
        }

        "Uncategorized".to_string()
    }

    fn extract_domain_folder(&self, host: &str) -> String {
        let mut parts: Vec<&str> = host.split('.').collect();

        // Remove 'www' if present
        if parts.first() == Some(&"www") {
            parts.remove(0);
        }

        // For domains like 'co.uk', 'com.au', etc., handle properly
        if parts.len() >= 3 && (parts[1] == "co" || parts[1] == "com" || parts[1] == "org") {
            // Keep the main domain and TLD
            format!("Domains/{}", parts[0])
        } else if parts.len() >= 2 {
            // Standard domain format - use second-to-last part (the actual domain)
            format!("Domains/{}", parts[parts.len() - 2])
        } else {
            format!("Domains/{}", host)
        }
    }

    fn categorize_by_content(&self, url: &str, title: &str) -> String {
        let content = format!("{} {}", url, title).to_lowercase();

        // Check for development-related keywords
        if self.contains_any(
            &content,
            &[
                "github",
                "gitlab",
                "stackoverflow",
                "developer",
                "api",
                "documentation",
                "docs",
            ],
        ) {
            return "Development".to_string();
        }

        // Check for social media
        if self.contains_any(
            &content,
            &["facebook", "twitter", "instagram", "linkedin", "social"],
        ) {
            return "Social".to_string();
        }

        // Check for shopping
        if self.contains_any(
            &content,
            &["amazon", "ebay", "shop", "store", "buy", "price"],
        ) {
            return "Shopping".to_string();
        }

        // Check for news
        if self.contains_any(&content, &["news", "article", "blog", "post", "wikipedia"]) {
            return "News & Reference".to_string();
        }

        // Check for entertainment
        if self.contains_any(&content, &["video", "movie", "music", "game", "stream"]) {
            return "Entertainment".to_string();
        }

        // Check for work/productivity
        if self.contains_any(
            &content,
            &["work", "office", "productivity", "tool", "service"],
        ) {
            return "Work".to_string();
        }

        "General".to_string()
    }

    fn categorize_by_date(&self, date_added: &Option<chrono::DateTime<chrono::Utc>>) -> String {
        if let Some(date) = date_added {
            let year = date.format("%Y").to_string();
            let month = date.format("%B").to_string();
            format!("By Date/{} {}", year, month)
        } else {
            "By Date/Unknown".to_string()
        }
    }

    fn contains_any(&self, text: &str, keywords: &[&str]) -> bool {
        keywords.iter().any(|&keyword| text.contains(keyword))
    }

    pub fn create_folder_structure<'a>(
        &self,
        bookmarks: &'a [Bookmark],
    ) -> HashMap<String, Vec<&'a Bookmark>> {
        let mut folder_map: HashMap<String, Vec<&Bookmark>> = HashMap::new();

        for bookmark in bookmarks {
            let folder = bookmark
                .folder
                .as_ref()
                .unwrap_or(&"Uncategorized".to_string())
                .clone();
            folder_map
                .entry(folder)
                .or_insert_with(Vec::new)
                .push(bookmark);
        }

        folder_map
    }

    pub fn generate_folder_summary(&self, bookmarks: &[Bookmark]) -> String {
        let folder_map = self.create_folder_structure(bookmarks);
        let mut summary = String::new();

        let mut sorted_folders: Vec<_> = folder_map.keys().collect();
        sorted_folders.sort();

        summary.push_str("# Bookmark Organization Summary\n\n");

        for folder in sorted_folders {
            let bookmarks = folder_map.get(folder).unwrap();
            summary.push_str(&format!(
                "## {} ({} bookmarks)\n\n",
                folder,
                bookmarks.len()
            ));

            for bookmark in bookmarks {
                if let Some(url) = &bookmark.url {
                    summary.push_str(&format!("- [{}]({})\n", bookmark.title, url));
                } else {
                    summary.push_str(&format!("- {}\n", bookmark.title));
                }
            }
            summary.push('\n');
        }

        summary
    }
}

pub fn create_automated_rules(bookmarks: &[Bookmark]) -> Vec<OrganizationRule> {
    let mut domain_counts: HashMap<String, usize> = HashMap::new();
    let mut title_patterns: HashMap<String, Vec<String>> = HashMap::new();

    // Count domain frequencies
    for bookmark in bookmarks {
        if let Some(url) = &bookmark.url {
            if let Ok(parsed) = Url::parse(url) {
                if let Some(host) = parsed.host_str() {
                    *domain_counts.entry(host.to_string()).or_insert(0) += 1;
                }
            }
        }

        // Extract common title patterns
        let title_words: Vec<String> = bookmark
            .title
            .split_whitespace()
            .map(|word| word.to_lowercase())
            .filter(|word| word.len() > 3)
            .collect();

        for word in title_words {
            title_patterns
                .entry(word.clone())
                .or_insert_with(Vec::new)
                .push(bookmark.title.clone());
        }
    }

    let mut rules = Vec::new();

    // Create rules for frequently used domains
    for (domain, count) in domain_counts {
        if count >= 5 {
            let folder_name = domain.split('.').next().unwrap_or(&domain).to_string();

            rules.push(OrganizationRule {
                name: format!("Auto: {}", domain),
                pattern: format!(r"{}", regex::escape(&domain)),
                folder: format!("Frequent/{}", folder_name),
                priority: 3,
            });
        }
    }

    rules
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_extraction() {
        let config = OrganizationConfig::default();
        let organizer = BookmarkOrganizer::new(config);

        assert_eq!(
            organizer.extract_domain_folder("www.github.com"),
            "Domains/github"
        );
        assert_eq!(
            organizer.extract_domain_folder("example.co.uk"),
            "Domains/example"
        );
        assert_eq!(
            organizer.extract_domain_folder("subdomain.example.com"),
            "Domains/example"
        );
    }

    #[test]
    fn test_content_categorization() {
        let config = OrganizationConfig::default();
        let organizer = BookmarkOrganizer::new(config);

        assert_eq!(
            organizer.categorize_by_content("https://github.com/user/repo", "GitHub Repo"),
            "Development"
        );

        assert_eq!(
            organizer.categorize_by_content("https://www.amazon.com/product", "Product on Amazon"),
            "Shopping"
        );
    }

    #[test]
    fn test_custom_rules() {
        let config = OrganizationConfig::default();
        let organizer = BookmarkOrganizer::new(config);

        let bookmark = Bookmark {
            id: "1".to_string(),
            title: "My Facebook Profile".to_string(),
            url: Some("https://www.facebook.com/profile".to_string()),
            folder: None,
            date_added: None,
            children: None,
        };

        let folder = organizer.determine_folder(&bookmark);
        assert_eq!(folder, "Social");
    }

    #[test]
    fn test_organize_preserves_existing() {
        let config = OrganizationConfig {
            preserve_existing: true,
            ..Default::default()
        };
        let organizer = BookmarkOrganizer::new(config);

        let bookmarks = vec![Bookmark {
            id: "1".to_string(),
            title: "GitHub".to_string(),
            url: Some("https://github.com".to_string()),
            folder: Some("My Folder".to_string()),
            date_added: None,
            children: None,
        }];

        let result = organizer.organize(bookmarks).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].folder.as_ref().unwrap().contains("My Folder"));
    }

    #[test]
    fn test_organize_replaces_folder() {
        let config = OrganizationConfig {
            preserve_existing: false,
            ..Default::default()
        };
        let organizer = BookmarkOrganizer::new(config);

        let bookmarks = vec![Bookmark {
            id: "1".to_string(),
            title: "GitHub".to_string(),
            url: Some("https://github.com".to_string()),
            folder: Some("Old Folder".to_string()),
            date_added: None,
            children: None,
        }];

        let result = organizer.organize(bookmarks).unwrap();
        assert_eq!(result.len(), 1);
        assert!(!result[0].folder.as_ref().unwrap().contains("Old Folder"));
    }

    #[test]
    fn test_multiple_categorization_rules() {
        let config = OrganizationConfig::default();
        let organizer = BookmarkOrganizer::new(config);

        // Test various URLs get categorized correctly
        let test_cases = vec![
            ("https://github.com", "GitHub", "Development"),
            ("https://twitter.com", "Twitter", "Social"),
            ("https://www.amazon.com/product", "Amazon", "Shopping"),
            ("https://nytimes.com/article", "News", "News & Reference"),
            ("https://youtube.com/watch", "Video", "Entertainment"),
        ];

        for (url, title, expected_category) in test_cases {
            let category = organizer.categorize_by_content(url, title);
            assert_eq!(
                category, expected_category,
                "Failed for URL: {}, Title: {}",
                url, title
            );
        }
    }

    #[test]
    fn test_domain_edge_cases() {
        let config = OrganizationConfig::default();
        let organizer = BookmarkOrganizer::new(config);

        // Test edge cases
        let test_cases = vec![
            ("example.com", "Domains/example"),
            ("sub.sub.example.com", "Domains/example"),
            ("www.example.com", "Domains/example"),
            ("github.io", "Domains/github"),
        ];

        for (host, expected) in test_cases {
            let result = organizer.extract_domain_folder(host);
            assert_eq!(result, expected, "Failed for host: {}", host);
        }
    }

    #[test]
    fn test_empty_bookmarks_list() {
        let config = OrganizationConfig::default();
        let organizer = BookmarkOrganizer::new(config);

        let bookmarks: Vec<Bookmark> = vec![];
        let result = organizer.organize(bookmarks).unwrap();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_bookmark_without_url() {
        let config = OrganizationConfig::default();
        let organizer = BookmarkOrganizer::new(config);

        let bookmarks = vec![Bookmark {
            id: "1".to_string(),
            title: "No URL".to_string(),
            url: None,
            folder: None,
            date_added: None,
            children: None,
        }];

        let result = organizer.organize(bookmarks).unwrap();
        assert_eq!(result.len(), 1);
        // Should categorize as Uncategorized
        assert_eq!(result[0].folder.as_ref().unwrap(), "Uncategorized");
    }
}
