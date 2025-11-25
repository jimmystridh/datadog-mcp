use reqwest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let api_key = std::env::var("DD_API_KEY")?;
    let app_key = std::env::var("DD_APP_KEY")?;

    println!("Testing direct API call...");
    println!(
        "API Key starts: {}...",
        &api_key.chars().take(10).collect::<String>()
    );

    let client = reqwest::Client::new();
    let response = client
        .get("https://api.datadoghq.eu/api/v1/monitor?page_size=1")
        .header("DD-API-KEY", &api_key)
        .header("DD-APPLICATION-KEY", &app_key)
        .send()
        .await?;

    println!("Status: {}", response.status());
    let body = response.text().await?;
    println!(
        "Response (first 300 chars): {}",
        &body.chars().take(300).collect::<String>()
    );

    Ok(())
}
