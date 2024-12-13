use alchemy_api::alchemy::Alchemy;
use std::env;
use warp::Filter;
use log::{info, error};
use dotenv::dotenv;
use reqwest;
use serde::Deserialize;

#[derive(serde::Serialize, Debug)]
struct NFT {
    id: String,
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize logging
    env_logger::init();
    info!("Starting NFT API server...");

    // Define the route for listing NFTs
    let list_nfts_route = warp::path!("nfts" / String)
        .and_then(getNFTsForOwner);

    // Start the server on port 8080
    info!("Server running on http://127.0.0.1:8080");
    warp::serve(list_nfts_route).run(([127, 0, 0, 1], 8080)).await;

    Ok(())
}

async fn getNFTsForOwner(owner: String) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Received request for address: {}", owner);

    // Get your Alchemy API key from environment variables
    let api_key = env::var("ALCHEMY_API_KEY")
        .expect("ALCHEMY_API_KEY must be set in .env file");

    // Fetch NFTs owned by the specified address
    let response = list_nfts(&api_key, &owner).await.map_err(|e| {
        error!("Failed to fetch NFTs: {:?}", e);
        warp::reject::not_found()
    })?;

    // Log and return the NFTs
    info!("NFTs found for address {}: {:?}", owner, response);
    Ok(warp::reply::json(&response))
}

async fn list_nfts(api_key: &String, owner: &String) -> Result<Vec<NFT>, Box<dyn std::error::Error>> {
    // Make an API call to Alchemy to fetch NFTs for the specified owner
    let url = format!("https://eth-mainnet.g.alchemy.com/v2/{}/getNFTs?owner={}",api_key, owner);
    //let url = format!("https://eth-mainnet.alchemyapi.io/v2/{}/getNFTs?owner={}", api_key, owner);
    
    // Use reqwest to make the GET request
    let res = reqwest::get(&url).await?;
    
    // Log the raw response body for debugging
    let body = res.text().await?;
    println!("Response Body: {}", body); // Log the response body

    // Deserialize the JSON response
    let res: AlchemyResponse = serde_json::from_str(&body)?;
    
    // Map the response to a Vec<NFT>
    let nfts = res.nfts.into_iter().map(|nft| NFT {
        id: nft.id,
        name: nft.name,
    }).collect();

    Ok(nfts)
}

// Define a struct for the API response (adjust fields as necessary)
#[derive(Deserialize)]
struct AlchemyResponse {
    nfts: Vec<NFTData>,
    totalCount: Option<u32>, // Optional field for total count
}

#[derive(Deserialize)]
struct NFTData {
    id: String,
    name: String,
}
