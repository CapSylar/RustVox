#[macro_use]
extern crate lazy_static;

pub mod engine;
mod threadpool;
mod ui;

pub use ui::UiRenderer;
pub use ui::DebugData;
pub use engine::world;
pub use engine::eye;
pub use engine::renderer::Renderer;