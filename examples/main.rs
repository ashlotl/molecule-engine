mod concurrency;
mod levels;
mod macros;
mod vulkan;

fn main() {
    // vulkan::how_to_use_vulkan::make_vulkan_test();
    macros::take_for_scope_basics::demonstrate_take_for_scope();
    // concurrency::synchronization_graph_tasks_basics::sync_test();
}