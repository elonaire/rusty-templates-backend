use axum::{
    extract::Multipart,
    http::StatusCode,
    response::IntoResponse,
};
use std::{fs::File, io::Write};

pub async fn upload(mut multipart: Multipart) -> impl IntoResponse {
    let upload_dir = "./src/uploads/";

    // Ensure the directory exists
    if let Err(e) = std::fs::create_dir_all(upload_dir) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create upload directory: {}", e),
        ).into_response();
    }

    while let Some(field) = multipart.next_field().await.unwrap_or_else(|_| None) {
        let mut field = field;

        // Extract field name and filename
        let filename = field
            .file_name()
            .map(|name| name.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let filepath = format!("{}/{}", upload_dir, filename);

        // Create and open the file for writing
        let mut file = match File::create(&filepath) {
            Ok(file) => file,
            Err(e) => return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create file: {}", e),
            ).into_response(),
        };

        // Read each chunk and write to the file
        while let Some(chunk) = match field.chunk().await {
            Ok(Some(chunk)) => Some(chunk),
            Ok(None) => None,
            Err(e) => {
                let _ = std::fs::remove_file(&filepath);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to read chunk: {}", e),
                ).into_response();
            }
        } {
            if let Err(e) = file.write_all(&chunk) {
                // Clean up file on error
                let _ = std::fs::remove_file(&filepath);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to write to file: {}", e),
                ).into_response();
            }
        }

        // Ensure file is successfully flushed
        if let Err(e) = file.flush() {
            // Clean up file on error
            let _ = std::fs::remove_file(&filepath);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to flush file: {}", e),
            ).into_response();
        }

        println!("Saved file to `{}`", filepath);
    }

    (StatusCode::OK, "Files successfully uploaded".to_string()).into_response()
}
