#![warn(clippy::all)]
#![allow(clippy::too_many_arguments)]

use __core::f32::consts::PI;
use glam::Vec3;
use imgui::*;
use imgui_sdl2_support::SdlPlatform;
use sdl2::{
    event::Event,
    keyboard,
    video::{GLProfile, SwapInterval},
};

#[macro_use]
extern crate lazy_static;

mod engine;
mod threadpool;
mod ui;

use engine::{eye::Eye, renderer::Renderer, world::World};

use std::time::Instant;
static MOUSE_SENSITIVITY: f32 = 0.05;

pub struct Telemetry {
    player_pos: Vec3,       // player position in absolute coordinates
    front: Vec3,          // front vector
    calculation_time: u128, // same as frame_time, but without waiting for the framebuffer swap
    frame_time: u128,       // should be 16ms on 60 Hz refresh rate
    num_triangles: usize,   // number of triangles on screen coming from chunks
    num_vertices: usize,    // number of vertices on screen coming from chunks
}

//TODO: refactor main
fn main() {
    // initialize SDL and its video subsystem
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_version(4, 3);
    gl_attr.set_context_profile(GLProfile::Core);

    // create a new window
    let window = video_subsystem
        .window("RustVox prototyping", 1700, 900)
        .allow_highdpi()
        .opengl()
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    // create a new opengl context and make it current
    let gl_context = window.gl_create_context().unwrap();

    // load up every opengl function, is this good ?
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as _);

    window.gl_make_current(&gl_context).unwrap();

    // enable vsync to cap framerate
    let res = window.subsystem().gl_set_swap_interval(SwapInterval::VSync);

    if let Err(s) = res {
        println!("error occured:{}", s);
    }

    let mut imgui = Context::create();
    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);

    // setup platform and renderer, and fonts to imgui
    let ui_renderer = ui::Ui::new(&mut imgui);

    let mut platform = SdlPlatform::init(&mut imgui);
    let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| {
        video_subsystem.gl_get_proc_address(s) as _
    });
    let mut event_pump = sdl.event_pump().unwrap();

    sdl.mouse().set_relative_mouse_mode(false);

    let mut state = Telemetry {
        player_pos: Vec3::ZERO,
        front: Vec3::ZERO,
        frame_time: 0,
        calculation_time: 0,
        num_triangles: 0,
        num_vertices: 0,
    };

    let mut voxel_world = World::new(Eye::new(
        PI / 4.0,
        1920.0 / 1080.0,
        0.1,
        500.0,
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 1.0, 0.0),
        1.0,
    ));

    let mut world_renderer = Renderer::new(&video_subsystem, &voxel_world);

    // FIXME: remove this
    let mut is_filled_mode = true; // opengl rendering mode
    let mut is_vsync_on = true;

    'main: loop {
        let start = Instant::now();
        // clear the frame

        // handle events here !!!
        for event in event_pump.poll_iter() {
            /* pass all events to imgui platfrom */
            platform.handle_event(&mut imgui, &event);

            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyDown { keycode: Some(s) , .. } => {
                        match s {
                            keyboard::Keycode::Num1 => sdl
                                .mouse()
                                .set_relative_mouse_mode(!sdl.mouse().relative_mouse_mode()),
                            keyboard::Keycode::Num2 => {
                                world_renderer.set_mode(if is_filled_mode {
                                    gl::LINE
                                } else {
                                    gl::FILL
                                });
                                is_filled_mode = !is_filled_mode
                            }
                            keyboard::Keycode::Num3 => {
                                window
                                    .subsystem()
                                    .gl_set_swap_interval(if is_vsync_on {
                                        SwapInterval::VSync
                                    } else {
                                        SwapInterval::Immediate
                                    })
                                    .expect("error setting swap interval");
                                is_vsync_on = !is_vsync_on
                            }
                            keyboard::Keycode::Escape => {
                                break 'main;
                            }
                            keyboard::Keycode::W => voxel_world.eye.move_forward(),
                            keyboard::Keycode::S => voxel_world.eye.move_backward(),
                            keyboard::Keycode::A => voxel_world.eye.strafe_left(),
                            keyboard::Keycode::D => voxel_world.eye.strafe_right(),
                            _ => (),
                        };
                }
                Event::MouseMotion {
                    xrel: x_rel,
                    yrel: y_rel,
                    ..
                } => voxel_world.eye.change_front_rel(
                    x_rel as f32 * MOUSE_SENSITIVITY,
                    y_rel as f32 * MOUSE_SENSITIVITY,
                ),
                _ => (),
            };
        }

        let calculation_start = Instant::now();
        voxel_world.update();
        // render the world
        world_renderer.draw_world(&voxel_world);
        let calculation_end = calculation_start.elapsed();

        // render the UI
        platform.prepare_frame(&mut imgui, &window, &event_pump);

        renderer.render(&mut imgui, |ui: &mut Ui| {
            ui_renderer.build_ui(ui, &mut state);
        });
        window.gl_swap_window();

        let end = start.elapsed();
        state.frame_time = end.as_micros();
        state.calculation_time = calculation_end.as_millis();
        voxel_world.set_stat(& mut state);
    }
}
