use std::time::Duration;

use tracing_subscriber::fmt::format::FmtSpan;

#[tracing::instrument]
async fn hello() {
    println!("hello");
    tokio::time::sleep(Duration::from_secs(1)).await;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // default tracing subscriber
    //let subscriber = tracing_subscriber::FmtSubscriber::new();

    use tracing_subscriber::fmt::format::FmtSpan;

    // configurable tracing subscriber
    let subscriber = tracing_subscriber::fmt()
        // output it in json
        // .json()
        // make it more compact abbreviated
        .compact()
        // display the source code file paths
        .with_file(true)
        // don't display the events target (module path in the filename)
        .with_target(false)
        // display the  source code line number
        .with_line_number(true)
        // display the thread ID the record was created on
        .with_thread_ids(true)
        // enable span events
        .with_span_events(FmtSpan::ENTER | FmtSpan::EXIT | FmtSpan::CLOSE)
        // build it
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    tracing::info!("starting  up");
    tracing::warn!("are you sure this is a good idea");
    tracing::error!("something went wrong");

    hello().await;
    Ok(())
}
