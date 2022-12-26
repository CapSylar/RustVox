use std::{ffi::{c_void, CStr}, mem::{size_of, self}, rc::Rc, cell::{RefCell}};
use gl::types;
use glam::{Vec3, Mat3, Mat4};
use image::EncodableLayout;
use sdl2::{VideoSubsystem};
use crate::DebugData;

use self::{opengl_abstractions::{shader::Shader}, csm::Csm, allocators::default_allocator::DefaultAllocator};
use super::{world::{World}, geometry::{mesh::Mesh, opengl_vertex::OpenglVertex}, sky::{sky_state::Sky, sky_renderer::SkyRenderer}};

pub mod opengl_abstractions;
pub mod csm;
pub mod allocators;

pub struct Renderer
{
    trans_ubo: u32,
    csm: Csm,
    shadow_fb: u32,
    default_shader : Shader,
    shadow_shader : Shader,
    sun_direction: Vec3,
    pub sky: Sky,
    sky_rend : SkyRenderer,

    // debug info
    debug_data: Rc<RefCell<DebugData>>,

    // timing info
    timers: [u32;3],
    timer_index: usize,
}

impl Renderer
{
    pub fn new(video_subsystem: &VideoSubsystem, world: &World, debug_info: &Rc<RefCell<DebugData>>) -> Self
    {
        // Setup
        // load up every opengl function, is this good ?
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as _);

        unsafe
        {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback( Some(error_callback) , std::ptr::null::<c_void>());
        }

        // setup the Opengl timers
        let timers: [u32;3] = [0;3];
        let timer_index = 0;
        unsafe
        {
            gl::GenQueries(timers.len() as i32, timers.as_ptr() as _);
            
            // fill them with dummy queries so they start with know values
            // because we do "triple buffering", if we set timer 0 initially, we would expect timer (0-1) % 3 to have it's value ready
            for index in timers
            {
                gl::BeginQuery(gl::TIME_ELAPSED, index);
                gl::EndQuery(gl::TIME_ELAPSED);
            }
        }

        unsafe
        {
            // Uniform Buffer Object for common transforms
            let mut trans_ubo = 0;
            gl::GenBuffers(1,&mut trans_ubo);
            gl::BindBuffer(gl::UNIFORM_BUFFER,trans_ubo);
            gl::BufferData(gl::UNIFORM_BUFFER,(size_of::<Mat4>()*3).try_into().unwrap(), std::ptr::null::<c_void>(), gl::DYNAMIC_DRAW);
            gl::BindBufferBase(gl::UNIFORM_BUFFER,0,trans_ubo);
            gl::BindBuffer(gl::UNIFORM_BUFFER,0);

            // TODO: this does not belong here
            // Load the Voxel Textures

            let tex_width = 64;
            let tex_height = 64;
            let dirt = image::open("rust-vox/textures/dirt.png").unwrap().flipv();
            let sand = image::open("rust-vox/textures/sand.png").unwrap().flipv();
            let water = image::open("rust-vox/textures/water.png").unwrap().flipv();

            let sand = sand.into_rgba8();
            let dirt = dirt.into_rgba8();
            let water = water.into_rgba8();

            let layer_count = 3; // only dirt and grass for now
            let mut texture_array = 0;
            gl::GenTextures(1, &mut texture_array);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, texture_array);
            gl::TexStorage3D(gl::TEXTURE_2D_ARRAY, 5, gl::RGBA8, tex_width, tex_height, layer_count);
            // upload one texture at a time
            gl::TexSubImage3D(gl::TEXTURE_2D_ARRAY, 0, 0, 0, 0, tex_width, tex_height, 1, gl::RGBA, gl::UNSIGNED_BYTE, dirt.as_bytes().as_ptr().cast());
            gl::TexSubImage3D(gl::TEXTURE_2D_ARRAY, 0, 0, 0, 1, tex_width, tex_height, 1, gl::RGBA, gl::UNSIGNED_BYTE, sand.as_bytes().as_ptr().cast());
            gl::TexSubImage3D(gl::TEXTURE_2D_ARRAY, 0, 0, 0, 2, tex_width, tex_height, 1, gl::RGBA, gl::UNSIGNED_BYTE, water.as_bytes().as_ptr().cast());

            gl::GenerateMipmap(gl::TEXTURE_2D_ARRAY);

            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_S, gl::REPEAT as _ );
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_T, gl::REPEAT as _ );
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MIN_FILTER, gl::NEAREST_MIPMAP_LINEAR as _ );
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY ,gl::TEXTURE_MAG_FILTER, gl::NEAREST as _ );

            gl::BindTexture(gl::TEXTURE_2D_ARRAY,0); // unbind

            // load the program
            let default_shader = Shader::new_from_vs_fs("rust-vox/shaders/default.vert",
             "rust-vox/shaders/default.frag" ).expect("Shader Error");
            
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
            gl::FrontFace(gl::CW);

            let shadow_shader =  Shader::new_from_vs_gs_fs("rust-vox/shaders/shadow.vert",
            "rust-vox/shaders/shadow.geom", "rust-vox/shaders/shadow.frag" ).expect("Shader Error");

            // generate a framebuffer for the shadow map
            let mut shadow_fb = 0;
            gl::GenFramebuffers(1, &mut shadow_fb);

            // initialise the Shadows
            let csm = Csm::new(2048,2048, &world.camera);

            gl::BindFramebuffer(gl::FRAMEBUFFER, shadow_fb);
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, csm.get_depth_texture_id(), 0);
            // need to explicitely mention that we will render no color on this framebuffer
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);
            // unbind
            gl::BindFramebuffer( gl::FRAMEBUFFER, 0);

            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, csm.get_depth_texture_id());

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, texture_array);

            let sky_rend = SkyRenderer::default();
            
            Self { trans_ubo, default_shader , shadow_shader , shadow_fb , csm, sun_direction: Vec3::ZERO, sky_rend,sky:Sky::default(), debug_data: debug_info.clone(),
                        timer_index, timers}
        }
    }

    pub fn draw_world(&mut self, world: &World)
    {   
        // run the start timer
        unsafe
        {
            gl::BeginQuery(gl::TIME_ELAPSED, self.timers[self.timer_index]);
        }

        let perspective = world.camera.get_persp_trans();
        let view = world.camera.get_look_at();
        let view_no_trans = Mat4::from_mat3(Mat3::from_mat4(view));

        let transforms: [Mat4;3] = [perspective,view,view_no_trans];
        // update global transformation UBO
        unsafe
        {
            gl::BindBuffer(gl::UNIFORM_BUFFER,self.trans_ubo);
            gl::BufferSubData(gl::UNIFORM_BUFFER,0,(size_of::<Mat4>()*transforms.len()).try_into().unwrap(),transforms.as_ptr() as _);
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0); // unbind
        }

        self.sky.update();
        self.sun_direction = self.sky.get_sun_direction();
        
        let sun_present = self.sky.is_sun_present();

        if sun_present // do not render shadows is the sun is not present
        {
            // PASS 1: render to the shadow map
            self.render_shadow(world);
        }

        let mut debug_data = self.debug_data.borrow_mut();

        unsafe
        {            
            // PASS 2: render the scene normally
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0); // bind default framebuffer
            gl::Viewport(0, 0, 1700 ,900);
            gl::ClearColor(0.25,0.5,0.88,1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);

            // render the sky
            self.sky_rend.render(&self.sky);

            self.default_shader.bind();

            self.default_shader.set_uniform1i("render_csm", i32::from(sun_present)).expect("error setting the sun present uniform");
            self.default_shader.set_uniform3fv("light_dir", &self.sun_direction).expect("error setting the light direction uniform");
            self.default_shader.set_uniform1i("voxel_textures", 0).expect("error binding texture altlas");
            self.default_shader.set_uniform1i("shadow_map", 1).expect("error setting the shadow_map array textures");

            // FIXME: setting the constant uniforms at every draw ?
            let cascades = self.csm.get_cascade_levels();
            self.default_shader.set_uniform1i("shadow_map", 1).expect("error setting the shadow map texture uniform");
            self.default_shader.set_uniform1i("cascade_count", cascades.len() as i32 ).expect("error setting the cascade count");
            self.default_shader.set_uniform_1fv("cascades", cascades).expect("error setting the cascades");

            debug_data.culled_chunks = Self::draw_geometry(world, &mut self.default_shader);
            Shader::unbind();
        }

        let mut timer: u64 = 0;
        unsafe
        {
            // run the end query
            gl::EndQuery(gl::TIME_ELAPSED);

            self.timer_index = (self.timer_index+1) % self.timers.len(); // advance timer index
            gl::GetQueryObjectui64v(self.timers[self.timer_index], gl::QUERY_RESULT, &mut timer as *mut u64);
        }

        // update debug data
        debug_data.draw_world_time = timer as f64 / 1000000.0; // in ms
    }

    fn draw_geometry(world: &World, shader: &mut Shader) -> usize
    {
        // draw each chunk's mesh
        let i = 0;

        // world.chunk_manager.allocator.render();

        // first draw all opaque meshes
        for chunk in world.chunk_manager.chunks_rendered.iter()
        {
            let chunk = chunk.borrow();

            let mesh = chunk.mesh.as_ref().unwrap();
            let vao = world.chunk_manager.allocator.get_vao(mesh.alloc_token.as_ref().unwrap());
            
            unsafe
            {
                vao.bind();

                gl::DrawElements(gl::TRIANGLES, chunk.get_num_opaque_indices() as _ , gl::UNSIGNED_INT, (chunk.get_num_trans_indices() * mem::size_of::<u32>()) as _ );

                vao.unbind();
            }
        }

        // then draw all transparent meshes
        for chunk in world.chunk_manager.chunks_rendered.iter()
        {
            let chunk = chunk.borrow();
            
            let mesh = chunk.mesh.as_ref().unwrap();
            let vao = world.chunk_manager.allocator.get_vao(mesh.alloc_token.as_ref().unwrap());
            
            unsafe
            {
                vao.bind();
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA); 

                gl::DrawElements(gl::TRIANGLES, chunk.get_num_trans_indices() as _  , gl::UNSIGNED_INT, 0 as _ );

                gl::Disable(gl::BLEND);

                vao.unbind();
            }  
        }

        i
    }

    /// Depth Only Render Pass
    /// Implements Cascaded Shadow Maps (CSM)
    fn render_shadow(&mut self, world: &World)
    {
        self.csm.update(&world.camera,self.sun_direction);

        self.shadow_shader.bind();

        unsafe
        {   
            gl::Viewport(0, 0, self.csm.get_width() , self.csm.get_height());
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.shadow_fb);
            gl::Clear(gl::DEPTH_BUFFER_BIT);
            gl::Disable(gl::CULL_FACE);
        }

        Self::draw_geometry(world,&mut self.shadow_shader);

        unsafe
        {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    pub fn draw_chunk<T: OpenglVertex> (allocator: &DefaultAllocator<T>, mesh: &Mesh<T>)
    {
        let vao = allocator.get_vao(mesh.alloc_token.as_ref().unwrap());
        unsafe
        {
            vao.bind();
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA); 

            gl::DrawElements(gl::TRIANGLES, mesh.indices.len() as _  , gl::UNSIGNED_INT, 0 as _ );

            gl::Disable(gl::BLEND);

            vao.unbind();
        }
    }

    pub fn draw_mesh<T: OpenglVertex> (allocator: &DefaultAllocator<T>, mesh: &Mesh<T>)
    {
        let vao = allocator.get_vao(mesh.alloc_token.as_ref().unwrap());
        unsafe
        {
            vao.bind();
            gl::DrawElements(gl::TRIANGLES, mesh.indices.len() as _  , gl::UNSIGNED_INT, 0 as _ );
            vao.unbind();
        }
    }

    // FIXME: duplicate code with function above, refactor
    pub fn draw_mesh_with_mode<T: OpenglVertex> (allocator: &DefaultAllocator<T>, mesh: &Mesh<T>, mode: types::GLenum )
    {
        let vao = allocator.get_vao(mesh.alloc_token.as_ref().unwrap());
        unsafe
        {
            vao.bind();
            gl::DrawElements(mode, mesh.indices.len() as _  , gl::UNSIGNED_INT, 0 as _ );
            vao.unbind();
        }
    }

    //FIXME: problematic interface
    pub fn set_mode(&mut self, mode: u32)
    {
        unsafe
        {
            gl::PolygonMode(gl::FRONT_AND_BACK, mode );
        }
    }

}

// error callback function for opengl
extern "system" fn error_callback ( _source : u32 , error_type : u32 , _id : u32 , _severity : u32 , _len : i32 , message: *const i8 , _user_param : *mut c_void )
{
    unsafe
    {
        if error_type == gl::DEBUG_TYPE_ERROR
        {
            let x = CStr::from_ptr(message).to_string_lossy().to_string();
            println!("ERROR CALLBACK: {}" , x);
        }
    }
}
