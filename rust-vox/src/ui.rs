use std::collections::VecDeque;

use glam::{Vec3, Vec2};
use imgui::{Condition, FontSource, Context, FontId, CollapsingHeader, Ui};
use imgui_opengl_renderer::Renderer;
use imgui_sdl2_support::SdlPlatform;
use sdl2::{VideoSubsystem, video::Window, EventPump};

use crate::{engine::{renderer::opengl_abstractions::{shader::Shader, vertex_array::{VertexLayout, VertexArray}}, geometry::{mesh::Mesh, opengl_vertex::{self, OpenglVertex}}, chunk_manager::ChunkManager}, world::World};

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

#[repr(C,packed)]
#[derive(Clone,Copy,Debug)]
struct UiVertex
{
    position: Vec2,
    uv: Vec2,
}

impl OpenglVertex for UiVertex
{
    fn get_layout() -> VertexLayout
    {
        let mut vertex_layout = VertexLayout::new();

        vertex_layout.push_f32(2);
        vertex_layout.push_f32(2);

        vertex_layout
    }
}

pub struct UiRenderer
{
    pub used_font: FontId,
    imgui_renderer: Renderer,
    ui_shader: Shader,
    cross_hair: Mesh<UiVertex>
}

impl UiRenderer
{
    pub fn new(video_subsystem: &VideoSubsystem, imgui_context: &mut Context, window: &Window) -> UiRenderer
    {
        // load the ImGui Font
        let used_font = imgui_context.fonts().add_font(&[FontSource::TtfData{
            data: include_bytes!("../resources/JetBrains.ttf"),
            size_pixels: 17.0 ,
            config: None,
        }]);

        // Setup Imgui UI
        let imgui_renderer = imgui_opengl_renderer::Renderer::new(imgui_context, |s| {
            video_subsystem.gl_get_proc_address(s) as _
        });

        // setup our UI shader
        let mut ui_shader = Shader::new_from_vs_fs("rust-vox/shaders/ui.vert", "rust-vox/shaders/ui.frag").expect("Error Creating UI Shader");

        // load UI texture
        let texture = image::open("rust-vox/textures/widgets.png").unwrap().flipv();
        let width = texture.width() as i32;
        let height = texture.height() as i32;
        let mut ui_texture = 0;

        unsafe
        {
            gl::GenTextures(1, &mut ui_texture);
            gl::ActiveTexture(gl::TEXTURE5);
            gl::BindTexture(gl::TEXTURE_2D,  ui_texture);

            gl::TexStorage2D(gl::TEXTURE_2D, 1, gl::RGBA8, width, height);
            gl::TexSubImage2D(gl::TEXTURE_2D, 0, 0,0, width, height, gl::RGBA, gl::UNSIGNED_BYTE, texture.as_bytes().as_ptr().cast());
            
            // gl::BindTexture(gl::TEXTURE_2D, 0); // Unbind
        }

        ui_shader.bind();
        ui_shader.set_uniform1i("ui_texture", 5).expect("error setting the UI sampler uniform");
        Shader::unbind();

        let size = window.size();
        let ratio = size.0 as f32 / size.1 as f32 ; // aspect ratio
        
        let mut cross_hair: Mesh<UiVertex> = Mesh::new();
        let uv_lower_left = Vec2::new(0.0/16.0,0.0/16.0);
        let offset = 1.0/16.0;

        cross_hair.add_quad(
        UiVertex{position:Vec2::new(-0.03,-0.03 * ratio), uv: uv_lower_left},
        UiVertex{position:Vec2::new(-0.03,0.03 * ratio), uv:uv_lower_left + Vec2::new(0.0,offset)},
        UiVertex{position:Vec2::new(0.03,0.03 * ratio), uv:uv_lower_left + Vec2::new(offset,offset)},
        UiVertex{position:Vec2::new(0.03,-0.03 * ratio), uv:uv_lower_left + Vec2::new(offset,0.0)}
        );

        cross_hair.upload();

        UiRenderer{ used_font, imgui_renderer, ui_shader, cross_hair }
    }

    /// Render the UI
    pub fn render(&mut self, voxel_world: &mut World, platform: &mut SdlPlatform, imgui_context: &mut Context, window: &Window, event_pump: &EventPump, debug_info: &mut DebugData)
    {
        // render the Imgui UI
        platform.prepare_frame(imgui_context, window, event_pump);

        self.imgui_renderer.render(imgui_context, |ui: &mut Ui| {
            self.build_ui(ui, debug_info, voxel_world);
        });

        // render our own UI
        self.ui_shader.bind();
        
        unsafe
        {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA); 
        }

        Self::draw_mesh(&self.cross_hair);

        unsafe
        {
            gl::Disable(gl::BLEND);
        }
        
        Shader::unbind();
    }
    
    //TODO: duplicate code
    pub fn draw_mesh<T> (mesh: &Mesh<T>)
    {
        unsafe
        {
            mesh.vao.as_ref().unwrap().bind();
            gl::DrawElements(gl::TRIANGLES, mesh.indices.len() as _  , gl::UNSIGNED_INT, 0 as _ );
            VertexArray::<T>::unbind();
        }
    }

    pub fn build_ui(&self, ui: &imgui::Ui , state:&mut DebugData, voxel_world: &mut World)
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
            ui.text(format!("player in chunk: {}", ChunkManager::get_chunk_pos(state.player_pos)));
            ui.text(format!("player position: {}", state.player_pos));
            ui.text(format!("look_at vector: {}", state.front));

            if ui.button("Rebuild World")
            {
                voxel_world.rebuild();
            }
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