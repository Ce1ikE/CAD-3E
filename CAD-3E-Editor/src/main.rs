use cad_3e_engine::CAD3EEngine;

fn main() {
    println!("Starting CAD-3E Engine...");

    // Create engine instance with 800x600 window at 60 FPS
    let engine = CAD3EEngine::new(800, 600, "CAD-3E Engine", 60);

    // Run the engine (this will handle the event loop)
    if let Err(e) = engine.run() {
        eprintln!("Engine error: {}", e);
    }
}
