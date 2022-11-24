#![warn(clippy::all)]
#![allow(clippy::too_many_arguments)]

use engine::{DebugData, world::World, camera::Camera, Renderer};
use glam::Vec3;
use imgui::Context;
use imgui_sdl2_support::SdlPlatform;
use sdl2::{
    event::Event,
    keyboard,
    video::{GLProfile, SwapInterval}, mouse::MouseButton,
};

use std::{time::Instant, f32::consts::PI};
static MOUSE_SENSITIVITY: f32 = 0.05;

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

    let mut imgui_context = Context::create();
    imgui_context.set_ini_filename(None);
    imgui_context.set_log_filename(None);

    // setup platform and renderer, and fonts to imgui
    let mut platform = SdlPlatform::init(&mut imgui_context);

    let mut ui_renderer = engine::UiRenderer::new(&video_subsystem, &mut imgui_context, &window);

    let mut event_pump = sdl.event_pump().unwrap();

    sdl.mouse().set_relative_mouse_mode(false);
    sdl.mouse().capture(false);

    let mut debug_data = DebugData::default();

    let mut voxel_world = World::new(Camera::new(
        PI / 4.0,
        1920.0 / 1080.0,
        0.1,
        500.0,
        Vec3::new(0.0, 60.0, 0.0),
        Vec3::new(1.0, 0.3, 0.0),
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
            platform.handle_event(&mut imgui_context, &event);

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
                            keyboard::Keycode::W => voxel_world.camera.move_forward(),
                            keyboard::Keycode::S => voxel_world.camera.move_backward(),
                            keyboard::Keycode::A => voxel_world.camera.strafe_left(),
                            keyboard::Keycode::D => voxel_world.camera.strafe_right(),
                            _ => (),
                        };
                }
                Event::MouseButtonDown { mouse_btn, .. } =>
                {
                    match mouse_btn
                    {
                        MouseButton::Right => { println!("Right Mouse Button Clicked");
                            voxel_world.destroy()},
                        MouseButton::Left => { println!("Left Mouse Button Clicked");
                            voxel_world.place()}
                        _ => (),
                    }
                },
                Event::MouseMotion {
                    xrel: x_rel,
                    yrel: y_rel,
                    ..
                } => 
                {
                    // ignore mouse movement if we are not in relative mode
                    if sdl.mouse().relative_mouse_mode()
                    {
                        voxel_world.camera.change_front_rel(
                            x_rel as f32 * MOUSE_SENSITIVITY,
                            y_rel as f32 * MOUSE_SENSITIVITY)
                    }
                },
                _ => (),
            };
        }

        let calculation_start = Instant::now();
        voxel_world.update();
        // render the world
        world_renderer.draw_world(&voxel_world);

        let calculation_end = calculation_start.elapsed();

        let end = start.elapsed();
        debug_data.frame_time = end.as_micros();
        debug_data.add_calculation_time(calculation_end.as_secs_f32());
        voxel_world.set_stat(& mut debug_data);

        // render the UI
        ui_renderer.render(&mut voxel_world, &mut platform, &mut imgui_context, &window, &event_pump, &mut debug_data);

        window.gl_swap_window();
    }
}
