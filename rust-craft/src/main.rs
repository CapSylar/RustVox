use __core::{mem::{size_of_val, size_of}, ffi::c_void};
use imgui::*;
use imgui_sdl2_support::SdlPlatform;
use sdl2::{
    event::Event,
    video::{GLProfile}
};

struct State
{
    example: u32,
}

fn main()
{
    // initialize SDL and its video subsystem
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_version(3, 3);
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

    let mut state = State{example:212};

    'main: loop
    {

    // handle events here !!!
    for event in event_pump.poll_iter() {
        /* pass all events to imgui platfrom */
        platform.handle_event(&mut imgui, &event);

        if let Event::Quit { .. } = event {
            break 'main;
        }
    }

    // clear the frame
    unsafe
    {
        gl::ClearColor(0.2,0.2,0.2,1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT)
    }


    unsafe
    {
        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        let mut vbo = 0;
        gl::GenBuffers(1, &mut vbo);

        type Vertex = [f32;3];
        const VERTICES : [Vertex;3] = [[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0]];

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        // load the vertex data
        gl::BufferData(gl::ARRAY_BUFFER, size_of_val(&VERTICES) as isize , VERTICES.as_ptr().cast() , gl::STATIC_DRAW);

        // specify the data format and buffer storage information for attribute index 0
        // this is specified for the currently bound VBO
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, size_of::<Vertex>().try_into().unwrap() , 0 as *const c_void );
        // vertex attributes are disabled by default 
        gl::EnableVertexAttribArray(0);
        // create program
        // first create the vertex shader
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);

        // vertex shader source
        //TODO: move this to a file and load it 
        const VERT_SHADER: &str = r##"
        #version 330 core
        layout (location = 0) in vec3 pos;
        void main()
        {
            gl_Position = vec4(pos.x, pos.y, pos.z, 1.0);
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
        void main()
        {
            color = vec4(1,0.2,0.2,1.0);
        }
        "##;

        // copy shader source into the specified shader object
        gl::ShaderSource(fragment_shader, 1, &(FRAG_SHADER.as_bytes().as_ptr().cast()), &(FRAG_SHADER.len().try_into().unwrap()));
        gl::CompileShader(fragment_shader);

        //TODO: refactor duplicate code 
        // get compilation status on the *hopefully* compiled fragment shader
        let mut success = 0;
        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);

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

        // mark as delete for later
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        // set program as current 
        gl::UseProgram(program);

        // actually draw something, yeahh
        gl::DrawArrays(gl::TRIANGLES, 0, 3);
    }   

    platform.prepare_frame(&mut imgui, &window, &event_pump);

    renderer.render( &mut imgui , | ui: &mut Ui |
        {
            let _roboto = ui.push_font(roboto);
            ui.window("hello imgui")
            .position([0.0,0.0], Condition::Always)
            .size([300.0, 300.0], Condition::FirstUseEver)
            .build(|| {
            ui.text_wrapped("Hello world!");
            ui.text("hello world");
            ui.separator();
            ui.text("hello world");
            ui.separator();
            ui.slider("u32 value", 0 , 1000 ,  &mut state.example);

            _roboto.pop();
        });        
        });
        
    window.gl_swap_window();
  }
}