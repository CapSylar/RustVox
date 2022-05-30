
use glam::{Vec3};
use imgui::*;
use imgui_sdl2_support::SdlPlatform;
use sdl2::{
    event::Event,
    video::{GLProfile}
};

mod ui;
mod engine;

use engine::{camera, renderer::Renderer};

use std::time::Instant;
static MOUSE_SENSITIVITY: f32 = 0.05;

struct State
{
    frame_time: u128,
}

fn main()
{
    // initialize SDL and its video subsystem
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_version(4, 3);
    gl_attr.set_context_profile(GLProfile::Core);

    // create a new window
    let window = video_subsystem
        .window("testing imGui", 1400, 900)
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
    let res = window.subsystem().gl_set_swap_interval(sdl2::video::SwapInterval::VSync);

    if let Err(s) = res
    {
        println!("error occured:{}" , s );
    }

    let mut imgui = Context::create();
    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);

    // setup platform and renderer, and fonts to imgui
    let ui_renderer = ui::Ui::new(&mut imgui);
    
    let mut platform = SdlPlatform::init(&mut imgui);
    let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| video_subsystem.gl_get_proc_address(s) as _);
    let mut event_pump = sdl.event_pump().unwrap();

    sdl.mouse().set_relative_mouse_mode(true);
    
    let mut state = State{frame_time:0};

    // camera transformations

    let mut camera = camera::Camera::new(Vec3::new(0.0,0.0,3.0),Vec3::new(0.0,0.0,-1.0),Vec3::new(0.0,1.0,0.0), 0.2);
    
    // let start_program = Instant::now();
    let mut world_renderer = Renderer::new(&video_subsystem);
    let mut is_filled_mode = true ; // opengl rendering mode

    'main: loop
    {
        let start = Instant::now();
        // clear the frame

        // handle events here !!!
        for event in event_pump.poll_iter() {
            /* pass all events to imgui platfrom */
            platform.handle_event(&mut imgui, &event);

            match event
            {
                Event::Quit { .. } => break 'main,
                Event::KeyDown{ keycode: code , .. } => 
                {
                    if let Some(s) = code 
                    {
                        match s
                        {
                            sdl2::keyboard::Keycode::Num1 => { sdl.mouse().set_relative_mouse_mode(!sdl.mouse().relative_mouse_mode()) }
                            sdl2::keyboard::Keycode::Num2 => 
                            {world_renderer.set_mode( if is_filled_mode {gl::LINE} else {gl::FILL} ); is_filled_mode = !is_filled_mode } ,
                            sdl2::keyboard::Keycode::Escape => { break 'main; }
                            sdl2::keyboard::Keycode::W => camera.move_forward(),
                            sdl2::keyboard::Keycode::S => camera.move_backward(),
                            sdl2::keyboard::Keycode::A => camera.strafe_left(),
                            sdl2::keyboard::Keycode::D => camera.strafe_right(),
                            _ => ()
                        };
                    }
                },
                Event::MouseMotion { xrel: x_rel , yrel: y_rel , .. } => 
                    camera.change_front_rel(x_rel as f32 * MOUSE_SENSITIVITY, y_rel as f32 * MOUSE_SENSITIVITY),
                _ => (),
            };
        }

        // render the world
        world_renderer.draw_world(&camera);

        platform.prepare_frame(&mut imgui, &window, &event_pump);

        renderer.render( &mut imgui , | ui: &mut Ui |
        {
            ui_renderer.build_ui(ui, &mut state );
        });        
        window.gl_swap_window();

        let end = start.elapsed();
        state.frame_time = end.as_micros();
    }
}
