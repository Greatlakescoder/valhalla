use clap::Parser;
use odin::{
    configuration::get_configuration,
    ollama::{create_system_prompt, OllamaRequest, OllamaResponse},
    os_tooling::SystemScanner,
    telemetry::{get_subscriber, init_subscriber},
};
use reqwest::Client;
use serde::Serialize;
use std::{error::Error, fs::File, io::BufWriter, path::Path};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL to send the request to
    #[arg(
        short,
        long,
        default_value = "http://ai-ollama.tail8c6aba.ts.net:11434/api/generate"
    )]
    url: String,

    /// Question to ask the model
    #[arg(short, long, default_value = "What is the origin of the name wesley")]
    query: String,
}

pub fn write_to_json<T: Serialize, P: AsRef<Path>>(data: &T, path: P) -> std::io::Result<()> {
    // Create file and wrap in buffered writer
    let file = File::create(path)?;
    let writer = BufWriter::new(file);

    // Serialize and write data
    serde_json::to_writer_pretty(writer, data)?;

    Ok(())
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
    let config = get_configuration().expect("Failed to read configuration.");
    let scanner = SystemScanner::build(&config.scanner);
    let results = scanner.scan_running_proccess()?;
    let tagged_results = scanner.tag_proccesses(results);
    write_to_json(
        &tagged_results,
        "/home/fiz/workbench/valhalla/data/output.json",
    )?;

    // for r in tagged_results {
    //     match r.to_json_string() {
    //         Ok(json) => {
    //             tracing::info!("{}", json);
    //         }
    //         Err(e) => eprintln!("Failed to serialize: {}", e),
    //     }
    // }

    // Start Chain of Thought

    // The amount of proccess on linux can be huge, we either need a way to filter them down or maybe have agent do it for us by only
    // passing pids and names?

    let system_prompt = create_system_prompt();

    for tp in tagged_results {
        let mut initial_prompt_input: String = String::from("");

        match tp.to_json_string() {
            Ok(json) => {
                tracing::debug!("{}", json);
                initial_prompt_input.push_str(&json)
            }
            Err(e) => eprintln!("Failed to serialize: {}", e),
        }

        let initial_prompt = format!("{},{}", system_prompt, initial_prompt_input);

        let request_body = OllamaRequest {
            model: "mistral".into(),
            prompt: initial_prompt,
            stream: false,
            options: { odin::ollama::Options { num_ctx: 20000 } },
        };
        let resp = match client.post(&args.url).json(&request_body).send().await {
            Ok(resp) => OllamaResponse::from_response(resp)
                .await
                .expect("Failed to talk to Ollama"),
            Err(err) => return Err(format!("Failed to send to request {err}").into()),
        };

        println!("Response: {}", &resp.response);
    }

    Ok(())
}
