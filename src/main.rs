use clap::Parser;
use odin_hackathon::{ollama::{OllamaRequest, OllamaResponse}, os_tooling::scan_running_proccess, telemetry::{get_subscriber, init_subscriber}};
use reqwest::Client;
use std::error::Error;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL to send the request to
    #[arg(short, long, default_value = "http://localhost:11434/api/generate")]
    url: String,

    /// Question to ask the model
    #[arg(short, long, default_value = "What is the origin of the name wesley")]
    query: String,
}

// Implementation to convert reqwest::Response into ApiResponse



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let client = Client::new();



    let subscriber = get_subscriber("odin".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    tracing::info!("Running Query");

    tracing::info!("Collecting Running Proccesses");
    let results = scan_running_proccess()?;

    let mut initial_prompt_input: String = String::from("");


    for r in results {
        match r.to_json_string() {
            Ok(json) => initial_prompt_input.push_str(&r.to_json_string().unwrap()),
            Err(e) => eprintln!("Failed to serialize: {}", e),
        }
    }
  

    // Start Chain of Though

    let initial_prompt = format!(
        "Analyze this json blob of processes for potential security concerns. {} \
        Give a brief assessment focused on obvious red flags.",
        initial_prompt_input
    );
    let request_body = OllamaRequest {
        model: "llama3.2".into(),
        prompt: initial_prompt_input,
        stream: false,
    };

    let resp = match client.post(&args.url).json(&request_body).send().await {
        Ok(resp) => OllamaResponse::from_response(resp)
            .await
            .expect("Failed to talk to Ollama"),
        Err(_) => return Err("Failed to send to request".into()),
    };

    // let mut request = match args.method.to_uppercase().as_str() {
    //     "GET" => client.get(&args.url),
    //     "POST" => client.post(&args.url),
    //     "PUT" => client.put(&args.url),
    //     "DELETE" => client.delete(&args.url),
    //     _ => return Err("Unsupported HTTP method".into()),
    // };

    // Add headers
    // for header in args.headers {
    //     let parts: Vec<&str> = header.split(':').collect();
    //     if parts.len() == 2 {
    //         request = request.header(parts[0].trim(), parts[1].trim());
    //     }
    // }

    // // Add body if provided
    // if let Some(body) = args.body {
    //     request = request.body(body);
    // }

    // Send request and get response

    println!("Response: {}", resp.response);

    Ok(())
}
