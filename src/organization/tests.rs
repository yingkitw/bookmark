use super::*;
use crate::exporter::Bookmark;

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
fn test_empty_bookmarks_list() {
    let config = OrganizationConfig::default();
    let organizer = BookmarkOrganizer::new(config);

    let bookmarks: Vec<Bookmark> = vec![];
    let result = organizer.organize(bookmarks).unwrap();
    assert!(result.is_empty());
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
    assert_eq!(result[0].folder.as_ref().unwrap(), "Uncategorized");
}

#[test]
fn test_domain_edge_cases() {
    let config = OrganizationConfig::default();
    let organizer = BookmarkOrganizer::new(config);

    assert_eq!(
        organizer.extract_domain_folder("localhost"),
        "Domains/localhost"
    );
    assert_eq!(
        organizer.extract_domain_folder("example.com"),
        "Domains/example"
    );
}

#[test]
fn test_multiple_categorization_rules() {
    let config = OrganizationConfig::default();
    let organizer = BookmarkOrganizer::new(config);

    // Social media should match custom rule first
    let bookmark = Bookmark {
        id: "1".to_string(),
        title: "Reddit".to_string(),
        url: Some("https://www.reddit.com/r/rust".to_string()),
        folder: None,
        date_added: None,
        children: None,
    };

    let folder = organizer.determine_folder(&bookmark);
    assert_eq!(folder, "Social");
}
