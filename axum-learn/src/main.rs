use mimalloc::MiMalloc;

use axum_learn::app;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    app::run().await
}
