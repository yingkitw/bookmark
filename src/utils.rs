//! Utility functions for file I/O and platform-specific operations

use anyhow::Result;
use std::fs;
use std::path::Path;

/// Open a file in the default application for the current platform
pub fn open_file(path: &Path) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(path).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("start").arg(path).spawn()?;
    }
    Ok(())
}

/// Create a redirect HTML file
pub fn create_redirect_html(output_path: &Path, target_url: &str) -> Result<()> {
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head><meta http-equiv="refresh" content="0;url={}"></head>
<body><p>Redirecting to <a href="{}">{}</a>...</p></body>
</html>"#,
        target_url, target_url, target_url
    );
    fs::write(output_path, html)?;
    Ok(())
}
