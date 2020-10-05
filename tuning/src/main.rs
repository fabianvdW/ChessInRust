use std::thread;
use tuning::*;

pub fn main() {
    let t = thread::Builder::new()
        .stack_size(12 * 1024 * 1024)
        .spawn(move || {
            actual_main();
        })
        .expect("Couldn't start thread");
    t.join().expect("Could not join thread");
}

pub fn actual_main() {
    //Step 1. Load all positions from a file. Those positions should already be the q-searched positions.
    let mut positions: Vec<TexelState> = Vec::with_capacity(1);
    tuning::loading::PositionLoader::new("D:/FenCollection/Andrews/E12.33-1M-D12-Resolved.epd", FileFormatSupported::EPD).load_texel_positions(&mut positions);
    println!("Loaded file with {} positions!", positions.len());
    tuning::loading::PositionLoader::new("D:/FenCollection/Andrews/E12.41-1M-D12-Resolved.epd", FileFormatSupported::EPD).load_texel_positions(&mut positions);
    println!("Loaded file with {} positions!", positions.len());
    let mut tuner = Tuner {
        k: 1.1155,
        positions,
        params: Parameters::default(),
    };
    println!("Start tuning for k");
    if OPTIMIZE_K {
        minimize_evaluation_error_fork(&mut tuner);
    }
    println!("Optimal K: {}", tuner.k);
    texel_tuning(&mut tuner);
}
