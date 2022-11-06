use std::collections::VecDeque;

use glam::Vec3;
use imgui::{Condition, FontSource, Context, FontId, CollapsingHeader};

pub struct DebugData {
    pub player_pos: Vec3,       // player position in absolute coordinates
    pub front: Vec3,          // front vector
    pub calculation_times: VecDeque<f32>, // same as frame_time, but without waiting for the framebuffer swap
    pub frame_time: u128,       // should be 16ms on 60 Hz refresh rate
    pub chunk_size_bytes: usize,       // bytes
    pub num_triangles: usize,   // number of triangles on screen coming from chunks
    pub num_vertices: usize,    // number of vertices on screen coming from chunks
}

impl DebugData
{
    pub fn default() -> Self
    {
        let mut calculation_times = VecDeque::new();
        calculation_times.resize(500, 0.5);

        DebugData { player_pos: Vec3::ZERO, front: Vec3::ZERO, frame_time: 0, num_triangles: 0, num_vertices: 0, calculation_times, chunk_size_bytes: 0 }
    }

    pub fn add_calculation_time(&mut self, value: f32)
    {
        if self.calculation_times.len() >= 500
        {self.calculation_times.pop_front();}

        self.calculation_times.push_back(value);
    }
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
    
    pub fn build_ui(&self, ui: &imgui::Ui , state:&mut DebugData)
    {
        let font = ui.push_font(self.used_font);
        ui.window("Tab")
        .position([0.0,0.0], Condition::Always)
        .size([500.0, 500.0], Condition::FirstUseEver)
        .build(|| {

        // Controls Section
        if CollapsingHeader::new("Controls")
        .default_open(false)
        .build(ui)
        {
            ui.text("Num1 to Toggle Mouse");
            ui.text("Num2 to Toggle Line Mode");
            ui.text("NUm3 to Toggle between Vsync Off/On");
        }

        // Debug Info Section
        if CollapsingHeader::new("Debug Info")
        .default_open(true)
        .build(ui)
        {
            ui.text(format!("player position: {}", state.player_pos));
            ui.text(format!("look_at vector: {}", state.front));
        }

        // Profiling Section
        if CollapsingHeader::new("Profiling")
        .default_open(true)
        .build(ui)
        {
            let vec: Vec<f32> = state.calculation_times.iter().cloned().collect();

            // build plot
            ui.plot_lines("Frame Times", &vec)
                .graph_size([0.0,60.0])
                .scale_min(0.0)
                .scale_max(60.0)
                .overlay_text(format!("{:.3} ms", state.calculation_times.back().unwrap()))
                .build();

            ui.text(format!("frame time: {:.2} us" , state.frame_time));
            ui.text(format!("FPS: {}", 1.0/(state.frame_time as f32 / 1000000.0) ));
            ui.text(format!("Chunk Triangles: {}", state.num_triangles));
            ui.text(format!("Chunk Vertices: {}", state.num_vertices));
            ui.text(format!("Chunk Level Info Storage: {:.2} MiBs", state.chunk_size_bytes as f32 / (1024f32 * 1024f32)));
        }

        font.pop();});
    }
}