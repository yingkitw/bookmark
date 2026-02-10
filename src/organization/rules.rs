use std::collections::HashMap;
use url::Url;

use crate::exporter::Bookmark;

use super::OrganizationRule;

/// Generate automated organization rules based on bookmark patterns
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
