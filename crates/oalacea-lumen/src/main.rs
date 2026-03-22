// Oalacea Lumen - CLI Entry Point
//
// AI-powered code analysis and test generation toolkit

use oalacea_lumen::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Cli::run().await
}
