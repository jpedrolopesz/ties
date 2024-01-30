use crate::cli::memory_analysis;

pub fn execute() {
    println!("Optimizing memory usage...");
    memory_analysis::analyze();
}
