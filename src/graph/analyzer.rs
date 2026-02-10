use std::collections::HashSet;

/// Extract meaningful tags from title and URL
pub fn extract_tags(title: &str, url: Option<&str>) -> Vec<String> {
    let stop_words: HashSet<&str> = [
        "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
        "of", "with", "by", "from", "is", "it", "this", "that", "are", "was",
        "be", "has", "had", "have", "do", "does", "did", "will", "would",
        "could", "should", "may", "might", "can", "not", "no", "so", "if",
        "my", "your", "his", "her", "its", "our", "their", "me", "him",
        "us", "them", "who", "what", "which", "when", "where", "how", "all",
        "each", "every", "both", "few", "more", "most", "other", "some",
        "such", "than", "too", "very", "just", "about", "up", "out", "new",
        "home", "page", "site", "web", "www", "http", "https", "com", "org",
        "net", "io", "html", "index", "default", "welcome",
    ].into_iter().collect();

    let mut tags = HashSet::new();

    // Extract from title words
    let words: Vec<String> = title
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3 && !stop_words.contains(w))
        .map(|w| w.to_string())
        .collect();

    for word in &words {
        tags.insert(word.clone());
    }

    // Extract path segments from URL
    if let Some(url_str) = url {
        if let Ok(parsed) = url::Url::parse(url_str) {
            for segment in parsed.path_segments().into_iter().flatten() {
                let seg = segment.to_lowercase();
                if seg.len() >= 3 && !stop_words.contains(seg.as_str()) {
                    // Remove file extensions
                    let clean = seg.split('.').next().unwrap_or(&seg);
                    if clean.len() >= 3 {
                        tags.insert(clean.to_string());
                    }
                }
            }
        }
    }

    tags.into_iter().collect()
}

/// Categorize a bookmark based on title, URL, and domain
pub fn categorize(title: &str, url: Option<&str>, domain: Option<&str>) -> String {
    let text = format!(
        "{} {}",
        title.to_lowercase(),
        url.unwrap_or("").to_lowercase()
    );
    let domain_lower = domain.unwrap_or("").to_lowercase();

    let categories: Vec<(&str, &[&str])> = vec![
        ("Development", &["github", "gitlab", "stackoverflow", "rust", "python", "javascript",
            "typescript", "golang", "java", "code", "programming", "developer", "api",
            "docker", "kubernetes", "npm", "crates", "pypi", "docs.rs", "dev.to",
            "compiler", "debug", "framework", "library", "sdk", "cli", "terminal"]),
        ("AI & ML", &["openai", "chatgpt", "huggingface", "tensorflow", "pytorch",
            "machine-learning", "deep-learning", "llm", "gpt", "claude", "gemini",
            "artificial-intelligence", "neural", "model", "training", "dataset",
            "watsonx", "granite", "copilot"]),
        ("Cloud & DevOps", &["aws", "azure", "gcloud", "cloud", "ibm.com", "heroku",
            "vercel", "netlify", "terraform", "ansible", "jenkins", "ci/cd",
            "devops", "infrastructure", "deploy", "container", "serverless"]),
        ("News & Media", &["news", "bbc", "cnn", "reuters", "nytimes", "medium",
            "blog", "article", "press", "journal", "magazine", "podcast"]),
        ("Social", &["twitter", "facebook", "linkedin", "reddit", "instagram",
            "youtube", "tiktok", "discord", "slack", "mastodon", "threads"]),
        ("Shopping", &["amazon", "ebay", "shop", "store", "buy", "price",
            "product", "cart", "checkout", "deal", "sale"]),
        ("Finance", &["bank", "finance", "invest", "stock", "crypto", "bitcoin",
            "trading", "portfolio", "payment", "paypal", "stripe"]),
        ("Education", &["learn", "course", "tutorial", "university", "edu",
            "academy", "school", "lecture", "study", "research", "paper",
            "arxiv", "scholar", "coursera", "udemy"]),
        ("Design", &["figma", "dribbble", "behance", "design", "ui", "ux",
            "css", "tailwind", "font", "icon", "color", "layout", "sketch"]),
        ("Reference", &["wikipedia", "docs", "documentation", "reference",
            "manual", "guide", "spec", "standard", "rfc", "mdn"]),
    ];

    for (category, keywords) in &categories {
        for keyword in *keywords {
            if text.contains(keyword) || domain_lower.contains(keyword) {
                return category.to_string();
            }
        }
    }

    "Other".to_string()
}

/// Compute Jaccard similarity between two tag sets
pub fn jaccard_similarity(tags_a: &HashSet<String>, tags_b: &HashSet<String>) -> f64 {
    if tags_a.is_empty() || tags_b.is_empty() {
        return 0.0;
    }
    let intersection = tags_a.intersection(tags_b).count();
    let union = tags_a.union(tags_b).count();
    if union > 0 {
        intersection as f64 / union as f64
    } else {
        0.0
    }
}

/// Extract domain from a URL, stripping "www." prefix
pub fn extract_domain(url: &str) -> Option<String> {
    match url::Url::parse(url) {
        Ok(parsed) => {
            let host = parsed.host_str()?;
            Some(host.strip_prefix("www.").unwrap_or(host).to_string())
        }
        Err(_) => None,
    }
}
