use super::*;
use crate::exporter::Bookmark;

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
        deduplicator
            .normalize_url("https://example.com/path/")
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

    let url1 = deduplicator
        .normalize_url("https://EXAMPLE.com/PATH")
        .unwrap();
    let url2 = deduplicator
        .normalize_url("https://example.com/path")
        .unwrap();

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
        (
            "https://example.com/path?foo=bar",
            "http://example.com/path",
        ),
        (
            "https://example.com/path#section",
            "http://example.com/path",
        ),
        ("https://EXAMPLE.COM/path", "http://example.com/path"),
        ("http://www.example.com/path/", "http://example.com/path"),
    ];

    for (input, expected) in test_cases {
        let result = deduplicator.normalize_url(input).unwrap();
        assert_eq!(result, expected, "Failed for input: {}", input);
    }
}
