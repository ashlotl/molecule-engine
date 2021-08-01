mod concurrency;
mod levels;
mod macros;
mod vulkan;

fn main() {
    vulkan::how_to_use_vulkan::make_vulkan_test();
    // concurrency::synchronization_graph_tasks_basics::sync_test();
}