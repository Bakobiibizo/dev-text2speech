use clap::{Parser, Subcommand};
use dev_text2speech::{app, backend, config::Config, TtsRequest};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Parser)]
#[command(version, about = "Bounded WhisperSpeech API and client")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}
#[derive(Subcommand)]
enum Command {
    Serve,
    Synthesize {
        text: String,
        #[arg(long)]
        voice: Option<String>,
        #[arg(long, default_value = "speech.wav")]
        output: String,
        #[arg(long, default_value = "http://127.0.0.1:7101")]
        url: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    dotenv::dotenv().ok();
    match Cli::parse().command.unwrap_or(Command::Serve) {
        Command::Serve => {
            let cfg = Config::load()?;
            let _child = if std::env::var("MANAGE_BACKEND").as_deref() == Ok("true") {
                backend::ensure_backend_running(&cfg.backend, &reqwest::Client::new()).await?
            } else {
                None
            };
            let addr: SocketAddr = format!("{}:{}", cfg.api_host, cfg.api_port).parse()?;
            tracing::info!(%addr, backend=%cfg.backend_url, "starting service");
            axum::serve(TcpListener::bind(addr).await?, app(cfg)).await?;
        }
        Command::Synthesize {
            text,
            voice,
            output,
            url,
        } => {
            let client = reqwest::Client::new();
            let mut request = client
                .post(format!("{}/v1/audio/speech", url.trim_end_matches('/')))
                .json(&TtsRequest { text, voice });
            if let Ok(key) = std::env::var("API_KEY") {
                request = request.bearer_auth(key);
            }
            let response = request.send().await?.error_for_status()?;
            tokio::fs::write(&output, response.bytes().await?).await?;
            println!("wrote {output}");
        }
    }
    Ok(())
}
