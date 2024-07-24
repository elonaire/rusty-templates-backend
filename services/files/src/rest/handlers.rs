use axum::{
    extract::{Multipart, Extension},
    http::StatusCode,
    response::IntoResponse,
};
use std::{fs::File, io::Write, sync::Arc, env};
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::graphql::schemas::general::UploadedFile;

pub async fn upload(Extension(db): Extension<Arc<Surreal<Client>>>, mut multipart: Multipart) -> impl IntoResponse {
    let upload_dir = env::var("FILE_UPLOADS_DIR")
    .expect("Missing the FILE_UPLOADS_DIR environment variable.");

    let mut total_size: u64 = 0;
    let mut filename = String::new();
    let mut mime_type = String::new();

    // Ensure the directory exists
    if let Err(e) = std::fs::create_dir_all(&upload_dir) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create upload directory: {}", e),
        ).into_response();
    }

    while let Some(field) = multipart.next_field().await.unwrap_or_else(|_| None) {
        let mut field = field;

        // Extract field name and filename
        filename = field
            .file_name()
            .map(|name| name.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let filepath = format!("{}/{}", &upload_dir, filename);
        // Extract the MIME type
        mime_type = field
            .content_type()
            .map(|mime| mime.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        println!("MIME type: {}", mime_type);

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
            total_size += chunk.len() as u64;
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

    let uploaded_file = UploadedFile {
        id: None,
        name: filename,
        size: total_size,
        mime_type,
        created_at: None
    };

    // Insert uploaded files into the database
    match db.create::<Vec<UploadedFile>>("file").content(uploaded_file.clone()).await {
        Ok(_result) => (StatusCode::CREATED, format!("Files successfully uploaded, size: {}MB", total_size / 1024 / 1024)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to insert into database: {}", e),
        ).into_response(),
    }
}
