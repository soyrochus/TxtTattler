use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    txttattler::run().await
}
