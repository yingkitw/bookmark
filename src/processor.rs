use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::deduplication::{
    find_potential_duplicates, BookmarkDeduplicator, DeduplicationConfig, DeduplicationResult,
};
use crate::exporter::{Bookmark, BrowserData};
use crate::organization::{BookmarkOrganizer, OrganizationConfig};

#[derive(Debug)]
pub struct ProcessingConfig {
    pub deduplication_config: DeduplicationConfig,
    pub organization_config: OrganizationConfig,
    pub dry_run: bool,
    pub backup_original: bool,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            deduplication_config: DeduplicationConfig::default(),
            organization_config: OrganizationConfig::default(),
            dry_run: false,
            backup_original: true,
        }
    }
}

#[derive(Debug)]
pub struct ProcessingResult {
    pub processed_bookmarks: Vec<Bookmark>,
    pub deduplication_result: Option<DeduplicationResult>,
    pub processing_summary: ProcessingSummary,
}

#[derive(Debug)]
pub struct ProcessingSummary {
    pub original_count: usize,
    pub final_count: usize,
    pub duplicates_removed: usize,
    pub folders_created: usize,
    pub processing_time: std::time::Duration,
    pub folder_distribution: HashMap<String, usize>,
}

pub struct BookmarkProcessor {
    config: ProcessingConfig,
}

impl BookmarkProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        Self { config }
    }

    pub fn process_browser_data(
        &self,
        browser_data: &[BrowserData],
    ) -> Result<Vec<ProcessingResult>> {
        let mut results = Vec::new();

        for data in browser_data {
            let result = self.process_bookmarks(data.bookmarks.as_ref().unwrap_or(&vec![]))?;
            results.push(result);
        }

        Ok(results)
    }

    pub fn process_bookmarks(&self, bookmarks: &[Bookmark]) -> Result<ProcessingResult> {
        let start_time = std::time::Instant::now();
        let original_count = bookmarks.len();

        // Step 1: Deduplicate bookmarks
        let (unique_bookmarks, deduplication_result) =
            if self.config.deduplication_config.normalize_urls {
                let deduplicator =
                    BookmarkDeduplicator::new(self.config.deduplication_config.clone());
                let result = deduplicator.deduplicate(bookmarks)?;
                (result.unique_bookmarks.clone(), Some(result))
            } else {
                (bookmarks.to_vec(), None)
            };

        // Step 2: Organize bookmarks into folders
        let organizer = BookmarkOrganizer::new(self.config.organization_config.clone());
        let organized_bookmarks = organizer.organize(unique_bookmarks)?;

        // Step 3: Create processing summary
        let folder_distribution: HashMap<String, usize> = organizer
            .create_folder_structure(&organized_bookmarks)
            .into_iter()
            .map(|(folder, bookmarks)| (folder, bookmarks.len()))
            .collect();

        let processing_time = start_time.elapsed();
        let final_count = organized_bookmarks.len();

        let processing_summary = ProcessingSummary {
            original_count,
            final_count,
            duplicates_removed: deduplication_result
                .as_ref()
                .map(|r| r.duplicates_removed)
                .unwrap_or(0),
            folders_created: folder_distribution.len(),
            processing_time,
            folder_distribution,
        };

        Ok(ProcessingResult {
            processed_bookmarks: organized_bookmarks,
            deduplication_result,
            processing_summary,
        })
    }

    pub fn merge_multiple_sources(&self, sources: &[Vec<Bookmark>]) -> Result<ProcessingResult> {
        // Combine all bookmarks from all sources
        let mut all_bookmarks = Vec::new();
        for source in sources {
            all_bookmarks.extend_from_slice(source);
        }

        // Add source information to bookmarks
        for (_source_index, _bookmark) in all_bookmarks.iter_mut().enumerate() {
            // We could add metadata about the source here if needed
        }

        // Process the combined bookmarks
        let result = self.process_bookmarks(&all_bookmarks)?;

        Ok(result)
    }

    pub fn export_processed_bookmarks(
        &self,
        bookmarks: &[Bookmark],
        output_path: &PathBuf,
    ) -> Result<()> {
        // Create BrowserData structure for the processed bookmarks
        let browser_data = BrowserData {
            browser: "Processed".to_string(),
            profile: "Deduplicated & Organized".to_string(),
            export_date: chrono::Utc::now(),
            bookmarks: Some(bookmarks.to_vec()),
            history: None,
            passwords: None,
        };

        let yaml_content = serde_yaml::to_string(&[browser_data])?;

        if self.config.dry_run {
            println!("DRY RUN: Would write to {}", output_path.display());
            println!(
                "Content preview:\n{}",
                &yaml_content[..yaml_content.len().min(500)]
            );
            return Ok(());
        }

        // Create backup if requested
        if self.config.backup_original && output_path.exists() {
            let backup_path = output_path.with_extension("yaml.bak");
            fs::copy(output_path, &backup_path)?;
            println!("Backup created: {}", backup_path.display());
        }

        fs::write(output_path, yaml_content)?;
        println!("Processed bookmarks exported to: {}", output_path.display());

        Ok(())
    }

    pub fn generate_report(&self, result: &ProcessingResult) -> String {
        let mut report = String::new();

        report.push_str("# Bookmark Processing Report\n\n");

        // Summary section
        report.push_str("## Summary\n\n");
        report.push_str(&format!(
            "- Original bookmarks: {}\n",
            result.processing_summary.original_count
        ));
        report.push_str(&format!(
            "- Final bookmarks: {}\n",
            result.processing_summary.final_count
        ));
        report.push_str(&format!(
            "- Duplicates removed: {}\n",
            result.processing_summary.duplicates_removed
        ));
        report.push_str(&format!(
            "- Folders created: {}\n",
            result.processing_summary.folders_created
        ));
        report.push_str(&format!(
            "- Processing time: {:?}\n\n",
            result.processing_summary.processing_time
        ));

        // Deduplication details
        if let Some(ref dedup_result) = result.deduplication_result {
            report.push_str("## Deduplication Details\n\n");
            report.push_str(&format!(
                "- Duplicates found: {}\n",
                dedup_result.duplicates_found
            ));
            report.push_str(&format!(
                "- Duplicates removed: {}\n",
                dedup_result.duplicates_removed
            ));

            if !dedup_result.merge_summary.is_empty() {
                report.push_str("\n### Merge Summary\n\n");
                for (url, count) in &dedup_result.merge_summary {
                    report.push_str(&format!("- {}: {} merged into 1\n", url, count));
                }
            }
            report.push('\n');
        }

        // Folder distribution
        report.push_str("## Folder Distribution\n\n");
        let mut sorted_folders: Vec<_> = result
            .processing_summary
            .folder_distribution
            .iter()
            .collect();
        sorted_folders.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count (descending)

        for (folder, count) in sorted_folders {
            report.push_str(&format!("- {}: {} bookmarks\n", folder, count));
        }

        report.push('\n');

        // Sample bookmarks from each folder
        report.push_str("## Sample Bookmarks by Folder\n\n");
        let folder_map = self.config.organization_config.folder_separator.clone();

        let organizer = BookmarkOrganizer::new(self.config.organization_config.clone());
        let folder_bookmarks = organizer.create_folder_structure(&result.processed_bookmarks);

        for (folder, bookmarks) in folder_bookmarks {
            report.push_str(&format!("### {} ({})\n\n", folder, bookmarks.len()));

            for (i, bookmark) in bookmarks.iter().take(3).enumerate() {
                if let Some(url) = &bookmark.url {
                    report.push_str(&format!("{}. [{}]({})\n", i + 1, bookmark.title, url));
                } else {
                    report.push_str(&format!("{}. {}\n", i + 1, bookmark.title));
                }
            }

            if bookmarks.len() > 3 {
                report.push_str(&format!("... and {} more\n\n", bookmarks.len() - 3));
            } else {
                report.push('\n');
            }
        }

        report
    }

    pub fn preview_changes(&self, bookmarks: &[Bookmark]) -> Result<()> {
        println!("# Preview of Processing Changes\n\n");

        // Sample bookmarks before and after
        println!("## Sample Bookmarks (Before → After)\n\n");

        for bookmark in bookmarks.iter().take(5) {
            println!("**Before:**");
            println!("  Title: {}", bookmark.title);
            println!(
                "  URL: {}",
                bookmark.url.as_ref().unwrap_or(&"N/A".to_string())
            );
            println!("  Folder: {:?}", bookmark.folder);

            // Process a single bookmark to show the change
            let processed = self.process_bookmarks(&[bookmark.clone()])?;
            if let Some(processed_bookmark) = processed.processed_bookmarks.first() {
                println!("**After:**");
                println!("  Title: {}", processed_bookmark.title);
                println!(
                    "  URL: {}",
                    processed_bookmark
                        .url
                        .as_ref()
                        .unwrap_or(&"N/A".to_string())
                );
                println!("  Folder: {:?}", processed_bookmark.folder);
            }
            println!();
        }

        // Show duplicate detection preview
        if self.config.deduplication_config.normalize_urls {
            let _deduplicator = BookmarkDeduplicator::new(self.config.deduplication_config.clone());
            let potential_duplicates = find_potential_duplicates(bookmarks)?;

            if !potential_duplicates.is_empty() {
                println!("## Potential Duplicates Found\n\n");
                for (bookmark1, bookmark2, similarity) in potential_duplicates.iter().take(5) {
                    println!("**Similarity: {:.2}%**", similarity * 100.0);
                    println!(
                        "1. {} ({})",
                        bookmark1.title,
                        bookmark1.url.as_ref().unwrap_or(&"N/A".to_string())
                    );
                    println!(
                        "2. {} ({})",
                        bookmark2.title,
                        bookmark2.url.as_ref().unwrap_or(&"N/A".to_string())
                    );
                    println!();
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_basic_processing() {
        let config = ProcessingConfig::default();
        let processor = BookmarkProcessor::new(config);

        let bookmarks = vec![
            Bookmark {
                id: "1".to_string(),
                title: "GitHub".to_string(),
                url: Some("https://github.com".to_string()),
                folder: None,
                date_added: Some(Utc::now()),
                children: None,
            },
            Bookmark {
                id: "2".to_string(),
                title: "GitHub Home".to_string(),
                url: Some("http://www.github.com".to_string()),
                folder: Some("Bookmarks".to_string()),
                date_added: Some(Utc::now()),
                children: None,
            },
        ];

        let result = processor.process_bookmarks(&bookmarks).unwrap();
        assert_eq!(result.processed_bookmarks.len(), 1); // Should be deduplicated
        assert_eq!(result.processing_summary.duplicates_removed, 1);
    }
}
