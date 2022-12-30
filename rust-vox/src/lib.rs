#[macro_use]
extern crate lazy_static;

pub mod engine;
mod threadpool;
pub mod generational_vec;
mod ui;

pub use ui::UiRenderer;
pub use ui::DebugData;
pub use engine::world;
pub use engine::camera;
pub use engine::renderer::Renderer;