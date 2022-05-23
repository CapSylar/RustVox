use std::ffi::CStr;

use __core::{mem::{size_of_val, size_of}, ffi::c_void, f32::consts::PI };
use glam::{Mat4, DMat4, Vec3};
use imgui::*;
use imgui_sdl2_support::SdlPlatform;
use sdl2::{
    event::Event,
    video::{GLProfile, self}
};
use std::time::Instant;

struct State
{
    red: f32,
    green: f32,
    blue: f32,
    rotation_x: f32,
    rotation_z : f32,
    depth_z: f32,
    view_x: f32,
    view_y : f32,
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

    let x = video_subsystem.gl_set_swap_interval(sdl2::video::SwapInterval::Immediate);

    if let Err(message) = x
    {
        println!("{}" , message);
    }
    
    // load up every opengl function, is this good ?
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as _);

    window.gl_make_current(&gl_context).unwrap();

    // enable vsync to cap framerate
    window.subsystem().gl_set_swap_interval(1).unwrap();

    let mut imgui = Context::create();
    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);

    // setup platform and renderer, and fonts to imgui

    // imgui.fonts().add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
    let roboto = imgui.fonts().add_font(&[FontSource::TtfData {
        data: include_bytes!("../resources/JetBrains.ttf"),
        size_pixels: 17.0 ,
        config: None,
    }]);

    let mut platform = SdlPlatform::init(&mut imgui);
    let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| video_subsystem.gl_get_proc_address(s) as _);
    let mut event_pump = sdl.event_pump().unwrap();

    let mut state = State{blue:1.0,green:1.0,red:1.0,rotation_x:0.0, rotation_z: 0.0 , depth_z:-3.0 ,frame_time:0,view_x:0.0,view_y:0.0};

    unsafe
    {
        gl::Enable(gl::DEBUG_OUTPUT);
        // TODO: what is extern "system" fn ? 
        gl::DebugMessageCallback( Some(error_callback) , 0 as *const c_void);
    }

    unsafe
    {
        type Vertex = [f32;3];
        type UV = [f32;2];
        
        let vertices:[f32; 180] = [ 
            -0.5, -0.5, -0.5,  0.0, 0.0,
             0.5, -0.5, -0.5,  1.0, 0.0,
             0.5,  0.5, -0.5,  1.0, 1.0,
             0.5,  0.5, -0.5,  1.0, 1.0,
            -0.5,  0.5, -0.5,  0.0, 1.0,
            -0.5, -0.5, -0.5,  0.0, 0.0,
    
            -0.5, -0.5,  0.5,  0.0, 0.0,
             0.5, -0.5,  0.5,  1.0, 0.0,
             0.5,  0.5,  0.5,  1.0, 1.0,
             0.5,  0.5,  0.5,  1.0, 1.0,
            -0.5,  0.5,  0.5,  0.0, 1.0,
            -0.5, -0.5,  0.5,  0.0, 0.0,
    
            -0.5,  0.5,  0.5,  1.0, 0.0,
            -0.5,  0.5, -0.5,  1.0, 1.0,
            -0.5, -0.5, -0.5,  0.0, 1.0,
            -0.5, -0.5, -0.5,  0.0, 1.0,
            -0.5, -0.5,  0.5,  0.0, 0.0,
            -0.5,  0.5,  0.5,  1.0, 0.0,
    
             0.5,  0.5,  0.5,  1.0, 0.0,
             0.5,  0.5, -0.5,  1.0, 1.0,
             0.5, -0.5, -0.5,  0.0, 1.0,
             0.5, -0.5, -0.5,  0.0, 1.0,
             0.5, -0.5,  0.5,  0.0, 0.0,
             0.5,  0.5,  0.5,  1.0, 0.0,
    
            -0.5, -0.5, -0.5,  0.0, 1.0,
             0.5, -0.5, -0.5,  1.0, 1.0,
             0.5, -0.5,  0.5,  1.0, 0.0,
             0.5, -0.5,  0.5,  1.0, 0.0,
            -0.5, -0.5,  0.5,  0.0, 0.0,
            -0.5, -0.5, -0.5,  0.0, 1.0,
    
            -0.5,  0.5, -0.5,  0.0, 1.0,
             0.5,  0.5, -0.5,  1.0, 1.0,
             0.5,  0.5,  0.5,  1.0, 0.0,
             0.5,  0.5,  0.5,  1.0, 0.0,
            -0.5,  0.5,  0.5,  0.0, 0.0,
            -0.5,  0.5, -0.5,  0.0, 1.0
        ];

        // let indices: [u32;6] = [0, 1, 3 , 1 ,2 ,3];

        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        let mut vbo = 0;
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, size_of_val(&vertices) as isize , vertices.as_ptr().cast() , gl::STATIC_DRAW);
        // specify the data format and buffer storage information for attribute index 0
        // this is specified for the currently bound VBO
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (5 * size_of::<f32>()).try_into().unwrap()  , 0 as *const c_void );
        // vertex attributes are disabled by default 
        gl::EnableVertexAttribArray(0);

        // let mut ebo = 0;
        // gl::GenBuffers(1, &mut ebo);
        // gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        // gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, size_of_val(&indices) as isize, indices.as_ptr().cast() , gl::STATIC_DRAW );

        // vertex attribute for the texture at location = 1
        gl::VertexAttribPointer(1, 2 , gl::FLOAT , gl::FALSE , (5 * size_of::<f32>()).try_into().unwrap() , (3 * size_of::<f32>()) as *const c_void  );
        gl::EnableVertexAttribArray(1);   

        // load texture
        let img = image::open("rust-vox/textures/quakeIII-arena-logo.jpeg").unwrap().flipv();
        let width = img.width();
        let height = img.height();
        let data = img.as_bytes();

        let mut texture1 = 0;
        gl::GenTextures(1, &mut texture1);
        
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, texture1);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _ );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _ );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as _ );
        gl::TexParameteri( gl::TEXTURE_2D , gl::TEXTURE_MAG_FILTER , gl::LINEAR as _ );

        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as _ , width.try_into().unwrap() , height.try_into().unwrap() ,
             0, gl::RGB as _ , gl::UNSIGNED_BYTE , data.as_ptr().cast() );
        gl::GenerateMipmap(gl::TEXTURE_2D);


        let img = image::open("rust-vox/textures/crate.jpeg").unwrap().flipv();
        let width = img.width();
        let height = img.height();
        let data = img.as_bytes();

        let mut texture2 = 0;
        gl::GenTextures(1, &mut texture2);

        gl::ActiveTexture(gl::TEXTURE1);
        gl::BindTexture(gl::TEXTURE_2D, texture2);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _ );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _ );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as _ );
        gl::TexParameteri( gl::TEXTURE_2D , gl::TEXTURE_MAG_FILTER , gl::LINEAR as _ );

        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as _ , width.try_into().unwrap(), height.try_into().unwrap(),
            0 , gl::RGB as _, gl::UNSIGNED_BYTE, data.as_ptr().cast());

        gl::GenerateMipmap(gl::TEXTURE_2D);

        
        // ------------------------------------------ END 

        gl::BindVertexArray(0); // unbind VAO
        gl::BindBuffer(gl::ARRAY_BUFFER , 0); // unbind currently bound buffer

        // create program
        // first create the vertex shader
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);

        // vertex shader source
        //TODO: move this to a file and load it 
        const VERT_SHADER: &str = r##"
        #version 330 core
        layout (location = 0) in vec3 pos;
        layout (location = 1) in vec2 in_tex_coord;

        uniform mat4 transform;

        out vec3 color;
        out vec2 tex_coord;
        
        void main()
        {
            gl_Position = transform * vec4(pos.x, pos.y, pos.z, 1.0);
            tex_coord = in_tex_coord;
        }
        "##;

        // copy shader source into the specified shader object
        gl::ShaderSource(vertex_shader, 1, &(VERT_SHADER.as_bytes().as_ptr().cast()), &(VERT_SHADER.len().try_into().unwrap()));
        gl::CompileShader(vertex_shader);

        // get compilation status on the *hopefully* compiled vertex shader
        let mut success = 0;
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);

        if success == 0
        {
            let mut log_length = 0_i32;
            gl::GetShaderiv(vertex_shader, gl::INFO_LOG_LENGTH, &mut log_length );

            let mut log_text: Vec<u8> = Vec::with_capacity(log_length.try_into().unwrap());
            // get shader log text
            gl::GetShaderInfoLog(vertex_shader, log_length , &mut log_length , log_text.as_mut_ptr().cast());

            panic!("Vertex Shader Compile Error: {}", String::from_utf8_lossy(&log_text) );
        }

        
        // same for the fragment shader
        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);

        // fragment shader source

        const FRAG_SHADER: &str = r##"
        #version 330 core
        
        out vec4 color;
        in vec2 tex_coord;

        uniform vec4 our_color;
        uniform sampler2D test_texture1;
        uniform sampler2D test_texture2;

        void main()
        {
            color = texture(test_texture1, tex_coord) * texture( test_texture2, tex_coord) * our_color ;
        }
        "##;

        // copy shader source into the specified shader object
        gl::ShaderSource(fragment_shader, 1, &(FRAG_SHADER.as_bytes().as_ptr().cast()), &(FRAG_SHADER.len().try_into().unwrap()));
        gl::CompileShader(fragment_shader);

        //TODO: refactor duplicate code 
        // get compilation status on the *hopefully* compiled fragment shader
        let mut success = 0;
        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);

        //FIXME: can't get message from shader info
        if success == 0
        {
            let mut log_length = 0_i32;
            gl::GetShaderiv(fragment_shader, gl::INFO_LOG_LENGTH, &mut log_length );

            let mut log_text: Vec<u8> = Vec::with_capacity(log_length.try_into().unwrap());
            // get shader log text
            gl::GetShaderInfoLog(fragment_shader, log_length , &mut log_length , log_text.as_mut_ptr().cast());

            panic!("Fragment Shader Compile Error: {}", String::from_utf8_lossy(&log_text) );
        }

        // attach shader and link program
        let program = gl::CreateProgram();
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);

        //TODO: refactor duplicate code

        let mut success = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);

        if success == 0
        {
            let mut log_length = 0_i32;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_length );

            let mut log_text: Vec<u8> = Vec::with_capacity(log_length.try_into().unwrap());
            // get shader log text
            gl::GetProgramInfoLog(program, log_length , &mut log_length , log_text.as_mut_ptr().cast());

            panic!("Program Link Error: {}", String::from_utf8_lossy(&log_text) );
        }

    gl::Enable(gl::DEPTH_TEST);

    // camera transformations

    let mut camera_pos = Vec3::new(0.0,0.0,3.0);
    let mut camera_front = Vec3::new(0.0,0.0,-1.0);
    let mut camera_up = Vec3::new(0.0,1.0,0.0);
    let camera_speed : f32 =  0.1;

    let start_program = Instant::now();

    'main: loop
    {
        let start = Instant::now();
        // clear the frame
        gl::ClearColor(0.2,0.2,0.2,1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        // handle events here !!!
        for event in event_pump.poll_iter() {
            /* pass all events to imgui platfrom */
            platform.handle_event(&mut imgui, &event);

            if let Event::Quit { .. } = event {
                break 'main;
            }
            else if let Event::KeyDown{ keycode : code , ..  } = event
            {
                if let Some(s) = code 
                {
                    match s
                    {
                        sdl2::keyboard::Keycode::W => { camera_pos += camera_front * camera_speed;},
                        sdl2::keyboard::Keycode::S => { camera_pos -= camera_front * camera_speed;},
                        sdl2::keyboard::Keycode::A => { camera_pos -= Vec3::cross(camera_front, camera_up).normalize() * camera_speed  },
                        sdl2::keyboard::Keycode::D => { camera_pos += Vec3::cross(camera_front, camera_up).normalize() * camera_speed } ,
                        _ => ()
                    };
                }
            }
        }

        // set program as current 
        gl::UseProgram(program);

        let location = gl::GetUniformLocation( program , b"our_color\0".as_ptr() as _ );
        gl::Uniform4f(location, state.red, state.green, state.blue, 1.0);

        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, texture1);
        gl::ActiveTexture(gl::TEXTURE1);
        gl::BindTexture(gl::TEXTURE_2D, texture2);
        
        let tex_location1 = gl::GetUniformLocation(program, b"test_texture1\0".as_ptr() as _ );
        gl::Uniform1i(tex_location1 , 0); // texture unit 0

        let tex_location2 = gl::GetUniformLocation( program , b"test_texture2\0".as_ptr() as _ );
        gl::Uniform1i( tex_location2 , 1); // texture unit 1 

        let transform_location = gl::GetUniformLocation( program , b"transform\0".as_ptr() as _ );

        let millis = start_program.elapsed().as_millis() as f32;

        let rotation = Mat4::from_rotation_y( millis/800.0 ) * Mat4::from_rotation_x( millis/600.0 );
        let translation = Mat4::from_translation(Vec3::new(0.0, 0.0, state.depth_z));
        let projection = Mat4::perspective_rh_gl(PI/4.0, 800.0/600.0, 0.1, 100.0);
        // camera matrix 
        let view = Mat4::look_at_rh( camera_pos, camera_pos + camera_front , camera_up );
        let trans =  projection * view * translation * rotation ;
        gl::UniformMatrix4fv( transform_location , 1 , gl::FALSE , trans.as_ref().as_ptr().cast() );

        gl::BindVertexArray(vao); // bind vao 
        // gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        // actually draw something, yeahh
        gl::DrawArrays( gl::TRIANGLES , 0 , 36 );
        // gl::DrawArrays(gl::TRIANGLES, 0, 3);
        gl::BindVertexArray(0);

        platform.prepare_frame(&mut imgui, &window, &event_pump);

        renderer.render( &mut imgui , | ui: &mut Ui |
        {
            let _roboto = ui.push_font(roboto);
            ui.window("Tab")
            .position([0.0,0.0], Condition::Always)
            .size([400.0, 300.0], Condition::FirstUseEver)
            .build(|| {
            ui.text_wrapped("Colors");
            ui.separator();
            ui.slider("Red", 0.0, 1.0, &mut state.red);
            ui.slider("Green", 0.0, 1.0, &mut state.green);
            ui.slider("Blue", 0.0, 1.0, &mut state.blue);
            ui.separator();
            ui.text("position settings");
            ui.separator();
            ui.slider("rotation in x" , 0.0 , PI , &mut state.rotation_x);
            ui.slider("rotation in z" , 0.0 , PI , &mut state.rotation_z);
            ui.slider("depth in z", 0.0 , -50.0 ,  &mut state.depth_z);
            ui.slider("view x" , -5.0 , 5.0 , &mut state.view_x);
            ui.slider("view y", -5.0 , 5.0 , &mut state.view_y);

            ui.text(format!("frame time: {}us" , state.frame_time));
        
            _roboto.pop();
        });
        });

        let end = start.elapsed();
        state.frame_time = end.as_micros();
        
        window.gl_swap_window();
    }
  }
}

extern "system" fn error_callback ( source : u32 , error_type : u32 , id : u32 , severity : u32 , len : i32 , message: *const i8 , user_param : *mut c_void )
{
    unsafe
    {
        if error_type == gl::DEBUG_TYPE_ERROR
        {
            let x = CStr::from_ptr(message).to_string_lossy().to_string();
            println!("error callback said: {}" , x);
        }
    }
}