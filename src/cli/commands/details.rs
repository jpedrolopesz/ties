use crate::cli::memory_analysis::{heap_analysis, stack_analysis};

pub async  fn execute() {
    println!("Detailed memory usage:");
    heap_analysis::analyze_heap().await;
    stack_analysis::analyze_stack();
}
