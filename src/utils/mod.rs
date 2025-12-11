//! Utilities for working with token metadata and IPFS uploads.
//!
//! This module provides functionality for creating and managing token metadata,
//! including uploading image and metadata to IPFS via the Pump.fun API.

pub mod transaction;

use isahc::AsyncReadResponseExt;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read};

/// Metadata structure for a token, matching the format expected by Pump.fun.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenMetadata {
    /// Name of the token
    pub name: String,
    /// Token symbol (e.g. "BTC")
    pub symbol: String,
    /// Description of the token
    pub description: String,
    /// IPFS URL of the token's image
    pub image: String,
    /// Whether to display the token's name
    pub show_name: bool,
    /// Creation timestamp/source
    pub created_on: String,
    /// Twitter handle
    pub twitter: Option<String>,
    /// Telegram handle
    pub telegram: Option<String>,
    /// Website URL
    pub website: Option<String>,
}

/// Response received after successfully uploading an image to IPFS.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageUploadResponse {
    /// IPFS URI where the image is stored
    pub image_uri: String,
}

/// Response received after successfully uploading token metadata.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenMetadataResponse {
    /// The uploaded token metadata
    pub metadata: TokenMetadata,
    /// IPFS URI where the metadata is stored
    pub metadata_uri: String,
}

/// Parameters for creating new token metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTokenMetadata {
    /// Name of the token
    pub name: String,
    /// Token symbol (e.g. "BTC")
    pub symbol: String,
    /// Description of the token
    pub description: String,
    /// Path to the token's image file
    pub file: String,
    /// Optional Twitter handle
    pub twitter: Option<String>,
    /// Optional Telegram group
    pub telegram: Option<String>,
    /// Optional website URL
    pub website: Option<String>,
}

/// Uploads an image file to IPFS via the Pump.fun API.
///
/// # Arguments
///
/// * `file_path` - Path to the image file to upload
///
/// # Returns
///
/// Returns a `Result` containing the IPFS image URI on success,
/// or an error if the upload fails.
async fn upload_image_to_ipfs(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let boundary = "------------------------f4d9c2e8b7a5310f";
    let mut body = Vec::new();

    // Append file part
    body.extend_from_slice(b"--");
    body.extend_from_slice(boundary.as_bytes());
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"file\"; filename=\"file\"\r\n");
    body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");

    // Read the file contents
    let mut file = File::open(file_path)?;
    let mut file_contents = Vec::new();
    file.read_to_end(&mut file_contents)?;
    body.extend_from_slice(&file_contents);

    // Close the boundary
    body.extend_from_slice(b"\r\n--");
    body.extend_from_slice(boundary.as_bytes());
    body.extend_from_slice(b"--\r\n");

    let client = isahc::HttpClient::new()?;
    let request = isahc::Request::builder()
        .method("POST")
        .uri("https://pump.fun/api/ipfs")
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", boundary),
        )
        .header("Content-Length", body.len() as u64)
        .body(isahc::AsyncBody::from(body))?;

    let mut response = client.send_async(request).await?;
    let text = response.text().await?;
    let json: ImageUploadResponse = serde_json::from_str(&text)?;

    Ok(json.image_uri)
}

/// Uploads token metadata JSON to IPFS via the Pump.fun API.
///
/// # Arguments
///
/// * `metadata` - Token metadata with all fields except the file path
/// * `image_uri` - IPFS URI of the uploaded image
///
/// # Returns
///
/// Returns a `Result` containing the `TokenMetadataResponse` with metadata IPFS URI on success,
/// or an error if the upload fails.
async fn upload_metadata_json(
    metadata: &CreateTokenMetadata,
    image_uri: &str,
) -> Result<TokenMetadataResponse, Box<dyn std::error::Error>> {
    let boundary = "------------------------f4d9c2e8b7a5310f";
    let mut body = Vec::new();

    // Helper function to append form data
    fn append_text_field(body: &mut Vec<u8>, boundary: &str, name: &str, value: &str) {
        body.extend_from_slice(b"--");
        body.extend_from_slice(boundary.as_bytes());
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"{}\"\r\n\r\n", name).as_bytes(),
        );
        body.extend_from_slice(value.as_bytes());
        body.extend_from_slice(b"\r\n");
    }

    // Append form fields (note: no file field)
    append_text_field(&mut body, boundary, "name", &metadata.name);
    append_text_field(&mut body, boundary, "symbol", &metadata.symbol);
    append_text_field(&mut body, boundary, "description", &metadata.description);
    append_text_field(&mut body, boundary, "image", image_uri);
    append_text_field(&mut body, boundary, "imageUri", image_uri);
    if let Some(twitter) = &metadata.twitter {
        append_text_field(&mut body, boundary, "twitter", twitter);
    }
    if let Some(telegram) = &metadata.telegram {
        append_text_field(&mut body, boundary, "telegram", telegram);
    }
    if let Some(website) = &metadata.website {
        append_text_field(&mut body, boundary, "website", website);
    }
    append_text_field(&mut body, boundary, "showName", "true");
    append_text_field(&mut body, boundary, "createdOn", "https://pump.fun");

    // Close the boundary
    body.extend_from_slice(b"--");
    body.extend_from_slice(boundary.as_bytes());
    body.extend_from_slice(b"--\r\n");

    let client = isahc::HttpClient::new()?;
    let request = isahc::Request::builder()
        .method("POST")
        .uri("https://pump.fun/api/ipfs")
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", boundary),
        )
        .header("Content-Length", body.len() as u64)
        .body(isahc::AsyncBody::from(body))?;

    let mut response = client.send_async(request).await?;
    let text = response.text().await?;
    let json: TokenMetadataResponse = serde_json::from_str(&text)?;

    Ok(json)
}

/// Creates and uploads token metadata to IPFS via the Pump.fun API in two steps.
///
/// This function first uploads the image file to IPFS, then uploads the metadata JSON
/// with the image URL included and the file field removed.
///
/// # Arguments
///
/// * `metadata` - Token metadata and image file information
///
/// # Returns
///
/// Returns a `Result` containing the `TokenMetadataResponse` with IPFS locations on success,
/// or an error if either upload step fails.
///
/// # Examples
///
/// ```rust,no_run
/// use pumpfun::utils::{CreateTokenMetadata, create_token_metadata};
///
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let metadata = CreateTokenMetadata {
///     name: "My Token".to_string(),
///     symbol: "MT".to_string(),
///     description: "A test token".to_string(),
///     file: "path/to/image.png".to_string(),
///     twitter: None,
///     telegram: None,
///     website: Some("https://example.com".to_string()),
/// };
///
/// let response = create_token_metadata(metadata).await?;
/// println!("Metadata URI: {}", response.metadata_uri);
/// # Ok(())
/// # }
/// ```
pub async fn create_token_metadata(
    metadata: CreateTokenMetadata,
) -> Result<TokenMetadataResponse, Box<dyn std::error::Error>> {
    // Step 1: Upload image file to IPFS
    let image_uri = upload_image_to_ipfs(&metadata.file).await?;

    // Step 2: Upload metadata JSON with image URL
    let response = upload_metadata_json(&metadata, &image_uri).await?;

    Ok(response)
}

/// Calculates the maximum amount to pay when buying tokens, accounting for slippage tolerance
///
/// # Arguments
/// * `amount` - The base amount in lamports (1 SOL = 1,000,000,000 lamports)
/// * `basis_points` - The slippage tolerance in basis points (1% = 100 basis points)
///
/// # Returns
/// The maximum amount to pay, including slippage tolerance
///
/// # Example
/// ```rust
/// use pumpfun::utils;
/// use solana_sdk::native_token::{sol_to_lamports, LAMPORTS_PER_SOL};
///
/// let amount = LAMPORTS_PER_SOL; // 1 SOL in lamports
/// let slippage = 100; // 1% slippage tolerance
///
/// let max_amount = utils::calculate_with_slippage_buy(amount, slippage);
/// assert_eq!(max_amount, sol_to_lamports(1.01f64)); // 1.01 SOL
/// ```
pub fn calculate_with_slippage_buy(amount: u64, basis_points: u64) -> u64 {
    amount + (amount * basis_points) / 10000
}

/// Calculates the minimum amount to receive when selling tokens, accounting for slippage tolerance
///
/// # Arguments
/// * `amount` - The base amount in lamports (1 SOL = 1,000,000,000 lamports)
/// * `basis_points` - The slippage tolerance in basis points (1% = 100 basis points)
///
/// # Returns
/// The minimum amount to receive, accounting for slippage tolerance
///
/// # Example
/// ```rust
/// use pumpfun::utils;
/// use solana_sdk::native_token::{sol_to_lamports, LAMPORTS_PER_SOL};
///
/// let amount = LAMPORTS_PER_SOL; // 1 SOL in lamports
/// let slippage = 100; // 1% slippage tolerance
///
/// let min_amount = utils::calculate_with_slippage_sell(amount, slippage);
/// assert_eq!(min_amount, sol_to_lamports(0.99f64)); // 0.99 SOL
/// ```
pub fn calculate_with_slippage_sell(amount: u64, basis_points: u64) -> u64 {
    amount - (amount * basis_points) / 10000
}