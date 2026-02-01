use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::deduplication::{DeduplicationConfig, MergeStrategy};
use crate::organization::{OrganizationConfig, OrganizationRule};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub deduplication: DeduplicationConfig,
    pub organization: OrganizationConfig,
    pub backup_enabled: bool,
    pub dry_run_by_default: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            deduplication: DeduplicationConfig::default(),
            organization: OrganizationConfig::default(),
            backup_enabled: true,
            dry_run_by_default: false,
        }
    }
}

impl AppConfig {
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            let default_config = AppConfig::default();
            default_config.save_to_file(path)?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(path)?;
        let config: AppConfig = if path.extension().and_then(|s| s.to_str()) == Some("json") {
            serde_json::from_str(&content)?
        } else {
            serde_yaml::from_str(&content)?
        };

        Ok(config)
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let content = if path.extension().and_then(|s| s.to_str()) == Some("json") {
            serde_json::to_string_pretty(self)?
        } else {
            serde_yaml::to_string(self)?
        };

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, content)?;
        Ok(())
    }

    pub fn get_default_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("bookmark-manager")
            .join("config.yaml")
    }

    pub fn load_or_create() -> Result<Self> {
        let config_path = Self::get_default_config_path();
        Self::load_from_file(&config_path)
    }

    pub fn create_sample_config(output_path: &PathBuf) -> Result<()> {
        let sample_config = AppConfig {
            deduplication: DeduplicationConfig {
                normalize_urls: true,
                ignore_query_params: true,
                ignore_fragment: true,
                ignore_www: true,
                ignore_protocol: true,
                case_sensitive: false,
                merge_strategy: MergeStrategy::MergeMetadata,
            },
            organization: OrganizationConfig {
                organize_by_domain: true,
                organize_by_category: true,
                organize_by_date: false,
                custom_rules: vec![
                    OrganizationRule {
                        name: "Development".to_string(),
                        pattern: r"(github|gitlab|bitbucket|stackoverflow|dev\.to|medium\.com)"
                            .to_string(),
                        folder: "Development".to_string(),
                        priority: 10,
                    },
                    OrganizationRule {
                        name: "Social Media".to_string(),
                        pattern: r"(facebook|twitter|x|instagram|linkedin|reddit|youtube|tiktok)"
                            .to_string(),
                        folder: "Social".to_string(),
                        priority: 9,
                    },
                    OrganizationRule {
                        name: "Shopping".to_string(),
                        pattern: r"(amazon|ebay|etsy|shopify|aliexpress|walmart|target)"
                            .to_string(),
                        folder: "Shopping".to_string(),
                        priority: 8,
                    },
                ],
                folder_separator: "/".to_string(),
                preserve_existing: true,
            },
            backup_enabled: true,
            dry_run_by_default: false,
        };

        sample_config.save_to_file(output_path)?;
        Ok(())
    }

    pub fn add_custom_rule(&mut self, rule: OrganizationRule) {
        // Check if rule with same name already exists
        if let Some(existing_rule) = self
            .organization
            .custom_rules
            .iter_mut()
            .find(|r| r.name == rule.name)
        {
            *existing_rule = rule;
        } else {
            self.organization.custom_rules.push(rule);
        }

        // Sort rules by priority
        self.organization
            .custom_rules
            .sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    pub fn remove_custom_rule(&mut self, rule_name: &str) -> Result<()> {
        let original_len = self.organization.custom_rules.len();
        self.organization
            .custom_rules
            .retain(|r| r.name != rule_name);

        if self.organization.custom_rules.len() == original_len {
            return Err(anyhow::anyhow!("Rule '{}' not found", rule_name));
        }

        Ok(())
    }

    pub fn list_rules(&self) -> Vec<&OrganizationRule> {
        self.organization.custom_rules.iter().collect()
    }

    pub fn validate_config(&self) -> Result<()> {
        // Validate deduplication config
        if self.deduplication.normalize_urls
            && !self.deduplication.ignore_query_params
            && !self.deduplication.ignore_fragment
        {
            // This is just a warning, not an error
            eprintln!("Warning: URL normalization is enabled but query parameters and fragments are not ignored");
        }

        // Validate organization rules
        for rule in &self.organization.custom_rules {
            if rule.folder.is_empty() {
                return Err(anyhow::anyhow!("Rule '{}' has empty folder", rule.name));
            }

            if rule.pattern.is_empty() {
                return Err(anyhow::anyhow!("Rule '{}' has empty pattern", rule.name));
            }

            // Test regex compilation
            if let Err(e) = regex::Regex::new(&rule.pattern) {
                return Err(anyhow::anyhow!(
                    "Invalid regex in rule '{}': {}",
                    rule.name,
                    e
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("test_config.yaml");

        config.save_to_file(&config_path).unwrap();
        let loaded_config = AppConfig::load_from_file(&config_path).unwrap();

        assert_eq!(
            config.deduplication.normalize_urls,
            loaded_config.deduplication.normalize_urls
        );
        assert_eq!(
            config.organization.organize_by_domain,
            loaded_config.organization.organize_by_domain
        );
    }

    #[test]
    fn test_custom_rule_management() {
        let mut config = AppConfig::default();
        let rule = OrganizationRule {
            name: "Test Rule".to_string(),
            pattern: r"test\.com".to_string(),
            folder: "Test".to_string(),
            priority: 5,
        };

        config.add_custom_rule(rule);
        assert_eq!(config.organization.custom_rules.len(), 7); // 6 default + 1 new

        config.remove_custom_rule("Test Rule").unwrap();
        assert_eq!(config.organization.custom_rules.len(), 6);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();

        // Valid config should pass
        assert!(config.validate_config().is_ok());

        // Invalid rule should fail
        let invalid_rule = OrganizationRule {
            name: "Invalid Rule".to_string(),
            pattern: r"[".to_string(), // Invalid regex
            folder: "Invalid".to_string(),
            priority: 1,
        };

        config.add_custom_rule(invalid_rule);
        assert!(config.validate_config().is_err());
    }
}
