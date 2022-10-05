use std::{ffi::{c_void, CStr}, mem::size_of};
use glam::{Vec3, Mat3, Mat4};
use sdl2::{VideoSubsystem};
use self::{opengl_abstractions::{shader::Shader, vertex_array::VertexArray}, csm::Csm};
use super::{world::{World}, mesh::{Mesh}, sky::{SkyRenderer, self}};

pub mod opengl_abstractions;
pub mod csm;
pub struct Renderer
{
    trans_ubo: u32,
    csm: Csm,
    shadow_fb: u32,
    default_shader : Shader,
    shadow_shader : Shader,
    _atlas_texture: u32,
    sun_direction: Vec3,
    sky_rend : SkyRenderer,
}

impl Renderer
{
    pub fn new(video_subsystem: &VideoSubsystem, world: &World) -> Self
    {
        // Setup
        // load up every opengl function, is this good ?
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as _);

        unsafe
        {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback( Some(error_callback) , std::ptr::null::<c_void>());
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

            let mut atlas_texture = 0;        
            // load texture atlas
            let img = image::open("rust-vox/textures/atlas.png").unwrap().flipv();
            let width = img.width();
            let height = img.height();
            let data = img.as_bytes();

            gl::GenTextures(1, &mut atlas_texture);
            
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, atlas_texture);

            gl::TexStorage2D(gl::TEXTURE_2D, 5, gl::RGBA8, width.try_into().unwrap(), height.try_into().unwrap());
            gl::TexSubImage2D(gl::TEXTURE_2D, 0, 0, 0, width.try_into().unwrap(), height.try_into().unwrap(), gl::RGBA, gl::UNSIGNED_BYTE, data.as_ptr().cast());

            gl::GenerateMipmap(gl::TEXTURE_2D);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _ );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _ );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST_MIPMAP_LINEAR as _ );
            gl::TexParameteri(gl::TEXTURE_2D ,gl::TEXTURE_MAG_FILTER, gl::NEAREST as _ );

            gl::BindVertexArray(0); // unbind VAO
            gl::BindBuffer(gl::ARRAY_BUFFER , 0); // unbind currently bound buffer

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
            let csm = Csm::new(2048,2048, &world.eye);

            gl::BindFramebuffer(gl::FRAMEBUFFER, shadow_fb);
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, csm.get_depth_texture_id(), 0);
            // need to explicitely mention that we will render no color on this framebuffer
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);
            // unbind
            gl::BindFramebuffer( gl::FRAMEBUFFER, 0);

            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, csm.get_depth_texture_id());

            let sky_rend = SkyRenderer::new();
            
            Self { trans_ubo, default_shader , shadow_shader , _atlas_texture: atlas_texture , shadow_fb , csm, sun_direction: Vec3::ZERO, sky_rend}
        }
    }

    pub fn draw_world(&mut self, world: &World)
    {   
        let perspective = world.eye.get_persp_trans();
        let view = world.eye.get_look_at();
        let view_no_trans = Mat4::from_mat3(Mat3::from_mat4(view));

        let transforms: [Mat4;3] = [perspective,view,view_no_trans];
        // update global transformation UBO
        unsafe
        {
            gl::BindBuffer(gl::UNIFORM_BUFFER,self.trans_ubo);
            gl::BufferSubData(gl::UNIFORM_BUFFER,0,(size_of::<Mat4>()*transforms.len()).try_into().unwrap(),transforms.as_ptr() as _);
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0); // unbind
        }

        unsafe
        {
            gl::Disable(gl::BLEND);
            gl::DepthFunc(gl::LEQUAL);
            // PASS 1: render to the shadow map
            self.render_shadow(world);
            
            // PASS 2: render the scene normally
            self.default_shader.bind();

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0); // bind default framebuffer
            gl::Viewport(0, 0, 1700 ,900);
            gl::ClearColor(0.25,0.5,0.88,1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            
            self.default_shader.set_uniform1i("texture_atlas", 0).expect("error binding texture altlas");
            self.default_shader.set_uniform1i("shadow_map", 1).expect("error setting the shadow_map array textures");

            // FIXME: setting the constant uniforms at every draw ?
            let cascades = self.csm.get_cascade_levels();
            self.default_shader.set_uniform1i("shadow_map", 1).expect("error setting the shadow map texture uniform");
            self.default_shader.set_uniform1i("cascade_count", cascades.len() as i32 ).expect("error setting the cascade count");
            self.default_shader.set_uniform_1fv("cascades", cascades).expect("error setting the cascades");

            Self::draw_geometry(world, &mut self.default_shader);
            Shader::unbind();

            self.sky_rend.render(world);
        }
    }

    fn draw_geometry(world: &World, shader: &mut Shader)
    {
        // draw each chunk's mesh
        for chunk in world.chunk_manager.get_chunks_to_render().iter()
        {
            let chunk = chunk.borrow_mut();
            if let Some(ref offset) = chunk.animation
            {
                shader.set_uniform3fv("animation_offset", &offset.current ).expect("error setting animation offset!");
            }
            else
            {
                shader.set_uniform3fv("animation_offset", &Vec3::ZERO).expect("error setting animation offset!");
            }

            Renderer::draw_mesh(chunk.mesh.as_ref().expect("mesh was not initialized!"));
        }
    }

    /// Depth Only Render Pass
    /// Implements Cascaded Shadow Maps (CSM)
    fn render_shadow(&mut self, world: &World)
    {
        self.csm.update(&world.eye,self.sun_direction);

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

    pub fn draw_mesh<T> (mesh: &Mesh<T>)
    {
        unsafe
        {
            mesh.vao.as_ref().unwrap().bind();
            gl::DrawElements(gl::TRIANGLES, mesh.indices.len() as _  , gl::UNSIGNED_INT, 0 as _ );
            VertexArray::<T>::unbind();
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
