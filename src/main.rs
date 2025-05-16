use crate::api::{ApiError, Client};
use std::env;

mod api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let inference_url = env::var("INFERENCE_URL").expect("INFERENCE_URL not set");
    let inference_key = env::var("INFERENCE_KEY").expect("INFERENCE_KEY not set");
    let inference_model = env::var("INFERENCE_MODEL").expect("INFERENCE_MODEL not set");

    let client = Client::new(inference_url, inference_key, inference_model);

    let prompt = "Write a short story about a robot learning to love.";

    match client.chat_completion(prompt).await {
        Ok(response) => {
            if let Some(choice) = response.choices.get(0) {
                println!("Generated Text:\n{}", choice.message.content);
            } else {
                println!("API call successful, but no choices returned.");
            }
        }
        Err(e) => {
            eprintln!("API Error: {}", e);
        }
    }

    Ok(())
}
