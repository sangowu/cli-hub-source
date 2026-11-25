use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

const MAX_ATTACHMENT_BYTES: usize = 20 * 1024 * 1024;
static NEXT_ATTACHMENT_ID: AtomicU64 = AtomicU64::new(0);

#[tauri::command]
pub fn save_composer_attachment(
    file_name: String,
    mime_type: String,
    bytes: Vec<u8>,
) -> Result<String, String> {
    if bytes.is_empty() {
        return Err("The pasted attachment is empty.".to_string());
    }
    if bytes.len() > MAX_ATTACHMENT_BYTES {
        return Err("The pasted attachment is larger than 20 MB.".to_string());
    }

    let extension = attachment_extension(&file_name, &mime_type);
    let directory = std::env::temp_dir()
        .join("cli-hub")
        .join("composer-attachments");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    let sequence = NEXT_ATTACHMENT_ID.fetch_add(1, Ordering::Relaxed);
    let path = directory.join(format!(
        "clipboard-{timestamp}-{}-{sequence}.{extension}",
        std::process::id(),
    ));

    fs::write(&path, bytes).map_err(|error| error.to_string())?;
    Ok(path.to_string_lossy().into_owned())
}

fn attachment_extension(file_name: &str, mime_type: &str) -> String {
    let from_name = PathBuf::from(file_name)
        .extension()
        .and_then(|extension| extension.to_str())
        .filter(|extension| {
            !extension.is_empty()
                && extension.len() <= 10
                && extension
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric())
        })
        .map(str::to_ascii_lowercase);

    from_name.unwrap_or_else(|| match mime_type.to_ascii_lowercase().as_str() {
        "image/jpeg" => "jpg".to_string(),
        "image/gif" => "gif".to_string(),
        "image/webp" => "webp".to_string(),
        "image/bmp" => "bmp".to_string(),
        "image/svg+xml" => "svg".to_string(),
        "application/pdf" => "pdf".to_string(),
        _ => "png".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::attachment_extension;

    #[test]
    fn keeps_safe_file_extension() {
        assert_eq!(attachment_extension("capture.WEBP", "image/png"), "webp");
    }

    #[test]
    fn rejects_unsafe_file_extension() {
        assert_eq!(
            attachment_extension("capture.bad-name", "image/jpeg"),
            "jpg"
        );
    }

    #[test]
    fn uses_png_for_unknown_clipboard_images() {
        assert_eq!(attachment_extension("image", "image/png"), "png");
    }
}
