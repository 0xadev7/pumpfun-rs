//! Utilities for working with token metadata and IPFS uploads.
//!
//! This module provides functionality for creating and managing token metadata,
//! including uploading image and metadata to IPFS via the Pump.fun API.

pub mod transaction;

use isahc::AsyncReadResponseExt;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read};
use anyhow::{Result, Context};

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

/// Creates and uploads token metadata to IPFS via the Pump.fun API.
///
/// This function takes token metadata and an image file, constructs a multipart form request,
/// and uploads it to the Pump.fun IPFS API endpoint. The metadata and image are stored on IPFS
/// and the function returns the IPFS locations.
///
/// # Arguments
///
/// * `metadata` - Token metadata and image file information
///
/// # Returns
///
/// Returns a `Result` containing the `TokenMetadataResponse` with IPFS locations on success,
/// or an error if the upload fails.
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
    // Step 1: Upload image to IPFS
    let image_url = upload_image_to_ipfs(&metadata.file).await
        .context("Failed to upload image to IPFS")?;
    
    tracing::info!("✅ Image uploaded to IPFS: {}", image_url);
    
    // Step 2: Upload JSON metadata to IPFS (including image URL)
    let metadata_response = upload_metadata_json_to_ipfs(
        &metadata.name,
        &metadata.symbol,
        &metadata.description,
        &image_url,
        metadata.twitter.as_deref(),
        metadata.telegram.as_deref(),
        metadata.website.as_deref(),
    ).await
        .context("Failed to upload metadata JSON to IPFS")?;
    
    tracing::info!("✅ Metadata uploaded to IPFS: {}", metadata_response.metadata_uri);
    
    Ok(metadata_response)
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


/// Step 1: Upload image file to IPFS
async fn upload_image_to_ipfs(file_path: &str) -> Result<String> {
    let boundary = "------------------------f4d9c2e8b7a5310f";
    let mut body = Vec::new();

    // Read the file contents
    let mut file = File::open(file_path)
        .context(format!("Failed to open image file: {}", file_path))?;
    let mut file_contents = Vec::new();
    file.read_to_end(&mut file_contents)
        .context("Failed to read image file")?;

    // Determine content type based on file extension
    let content_type = if file_path.ends_with(".png") {
        "image/png"
    } else if file_path.ends_with(".jpg") || file_path.ends_with(".jpeg") {
        "image/jpeg"
    } else if file_path.ends_with(".gif") {
        "image/gif"
    } else {
        "application/octet-stream"
    };

    // Build multipart form data with file only
    body.extend_from_slice(b"--");
    body.extend_from_slice(boundary.as_bytes());
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"file\"; filename=\"image\"\r\n").as_bytes(),
    );
    body.extend_from_slice(format!("Content-Type: {}\r\n\r\n", content_type).as_bytes());
    body.extend_from_slice(&file_contents);
    body.extend_from_slice(b"\r\n--");
    body.extend_from_slice(boundary.as_bytes());
    body.extend_from_slice(b"--\r\n");

    // Send request to pump.fun IPFS endpoint
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
    
    let json: ImageUploadResponse = serde_json::from_str(&text)
        .context(format!("Failed to parse image upload response: {}", text))?;

    Ok(json.image)
}

/// Step 2: Upload JSON metadata to IPFS (with image URL)
async fn upload_metadata_json_to_ipfs(
    name: &str,
    symbol: &str,
    description: &str,
    image_url: &str,
    twitter: Option<&str>,
    telegram: Option<&str>,
    website: Option<&str>,
) -> Result<MetadataUploadResponse> {
    // Build JSON metadata object
    let mut metadata_json = serde_json::json!({
        "name": name,
        "symbol": symbol,
        "description": description,
        "image": image_url,
        "showName": true,
        "createdOn": "https://pump.fun",
    });

    // Add optional fields
    if let Some(tw) = twitter {
        if !tw.is_empty() {
            metadata_json["twitter"] = serde_json::json!(tw);
        }
    }
    if let Some(tg) = telegram {
        if !tg.is_empty() {
            metadata_json["telegram"] = serde_json::json!(tg);
        }
    }
    if let Some(ws) = website {
        if !ws.is_empty() {
            metadata_json["website"] = serde_json::json!(ws);
        }
    }

    let boundary = "------------------------f4d9c2e8b7a5310f";
    let mut body = Vec::new();

    // Append JSON as file field
    let json_string = serde_json::to_string(&metadata_json)?;
    
    body.extend_from_slice(b"--");
    body.extend_from_slice(boundary.as_bytes());
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"file\"; filename=\"metadata.json\"\r\n",
    );
    body.extend_from_slice(b"Content-Type: application/json\r\n\r\n");
    body.extend_from_slice(json_string.as_bytes());
    body.extend_from_slice(b"\r\n--");
    body.extend_from_slice(boundary.as_bytes());
    body.extend_from_slice(b"--\r\n");

    // Send request to pump.fun IPFS endpoint
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
    
    let json: MetadataUploadResponse = serde_json::from_str(&text)
        .context(format!("Failed to parse metadata upload response: {}", text))?;

    Ok(json)
}
