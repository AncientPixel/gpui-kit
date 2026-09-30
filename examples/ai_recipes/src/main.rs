// TEMP (gpui-fast): its macros expand to `::gpui`; resolve that to the Kit.
extern crate gpui_kit as gpui;

fn main() {
    gpui_kit_recipes::bootstrap::run();
}
