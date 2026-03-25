use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use transactor::engine::Engine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Create a single, shared global Engine wrapped in Arc and Mutex
    let engine = Arc::new(Mutex::new(Engine::default()));

    let listener: TcpListener = TcpListener::bind("127.0.0.1:8080").await?;
    log::info!("Server running. Send a CSV file via netcat:\nnc 127.0.0.1 8080 < transactions.csv");

    // We use a loop, but we wrap the inside in `tokio::select!` so that we can eventually escape it
    // through graceful process termination (e.g. Ctrl+C or kill -INT from the OS)
    loop {
        tokio::select! {
            // Task A: Wait for a new TCP connection
            accept_result = listener.accept() => {
                let (tokio_stream, addr) = accept_result?;
                log::info!("Accepted connection from: {}", addr);

                // Clone the Arc so this specific connection has access to the Engine
                let engine_clone = Arc::clone(&engine);

                tokio::spawn(async move {
                    let std_stream = match tokio_stream.into_std() {
                        Ok(s) => s,
                        Err(e) => { log::error!("Failed to convert stream: {}", e); return; }
                    };

                    if let Err(e) = std_stream.set_nonblocking(false) {
                        log::error!("Failed to set blocking mode: {}", e); return;
                    }

                    // Move the heavy lifting to a blocking thread
                    let result = tokio::task::spawn_blocking(move || {
                        // Lock the mutex to get exclusive mutable access to the engine
                        let mut locked_engine = engine_clone.lock().unwrap();

                        if let Err(e) = locked_engine.load_transactions_from_reader(std_stream) {
                            log::error!("Engine error for {}: {}", addr, e);
                        }
                    }).await;

                    if result.is_err() { log::error!("Task panicked while processing {}", addr); }
                    else { log::info!("Finished processing connection from {}", addr); }
                });
            }

            // Task B: Wait for the user to terminate the server (Ctrl+C or SIGINT from the OS)
            _ = tokio::signal::ctrl_c() => {
                log::info!("Ctrl+C or SIGINT received! Shutting down the server gracefully...");
                // Break out of the infinite loop
                break;
            }
        }
    }

    // The loop is over. The server is no longer accepting connections.Now we can output the final,
    // global state.
    let final_engine = engine.lock().unwrap();
    let _ = final_engine.output_accounts_into_stdout();

    Ok(())
}
