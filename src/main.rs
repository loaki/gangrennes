#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    gangrennes::startup::run().await
}
