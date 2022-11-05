use glam::Vec3;
use imgui::{Condition, FontSource, Context, FontId};

pub struct Telemetry {
    pub player_pos: Vec3,       // player position in absolute coordinates
    pub front: Vec3,          // front vector
    pub calculation_time: u128, // same as frame_time, but without waiting for the framebuffer swap
    pub frame_time: u128,       // should be 16ms on 60 Hz refresh rate
    pub num_triangles: usize,   // number of triangles on screen coming from chunks
    pub num_vertices: usize,    // number of vertices on screen coming from chunks
}

pub struct Ui
{
    pub used_font: FontId,
}

impl Ui
{
    pub fn new(imgui: &mut Context) -> Ui
    {
        let used_font = imgui.fonts().add_font(&[FontSource::TtfData{
            data: include_bytes!("../resources/JetBrains.ttf"),
            size_pixels: 17.0 ,
            config: None,
        }]);

        Ui{ used_font }
    }
    
    pub fn build_ui(&self, ui: & imgui::Ui , state:&mut Telemetry)
    {
        let font = ui.push_font(self.used_font);
        ui.window("Tab")
        .position([0.0,0.0], Condition::Always)
        .size([500.0, 500.0], Condition::FirstUseEver)
        .build(|| {
        ui.text_wrapped("Controls");
        ui.separator();
        ui.text("Num1 to Toggle Mouse");
        ui.text("Num2 to Toggle Line Mode");
        ui.text("NUm3 to Toggle between Vsync Off/On");
        ui.separator();
        ui.text_wrapped("Info");
        ui.text(format!("player position: {}", state.player_pos));
        ui.text(format!("look_at vector: {}", state.front));
        ui.separator();
        ui.text_wrapped("Performance");
        ui.text(format!("calculation time: {}ms", state.calculation_time));
        ui.text(format!("frame time: {}us" , state.frame_time));
        ui.text(format!("FPS: {}", 1.0/(state.frame_time as f32 / 1000000.0) ));
        ui.text(format!("Chunk Triangles: {}", state.num_triangles));
        ui.text(format!("Chunk Vertices: {}", state.num_vertices));
        font.pop();});
    }
}