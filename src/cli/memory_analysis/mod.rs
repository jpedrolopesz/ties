pub mod heap_analysis;
pub mod stack_analysis;
pub mod data_structures;
pub mod report;

pub fn analyze() {
    println!("Analyzing memory...");
    heap_analysis::analyze_heap();
    stack_analysis::analyze_stack();
}
