use std::{ffi::{c_void, CStr}, f32::consts::PI};
use glam::{Mat4, Vec3};
use sdl2::{VideoSubsystem};

pub mod vertex_buffer;
pub mod index_buffer;
pub mod vertex_array;
pub mod shader;

use self::{vertex_array::{VertexArray}, shader::Shader};

use super::{world::World, mesh::Mesh, voxel::Voxel};

pub struct Renderer
{
    shadow_fb: u32,
    default_shader : Shader,
    shadow_shader : Shader,
    texture1: u32,
    depth_texture: u32,
    sun: Mesh,
}

impl Renderer
{
    pub fn new(video_subsystem: &VideoSubsystem) -> Renderer
    {
        // Setup
        // load up every opengl function, is this good ?
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as _);

        unsafe
        {
            gl::Enable(gl::DEBUG_OUTPUT);
            // TODO: what is extern "system" fn ? 
            gl::DebugMessageCallback( Some(error_callback) , 0 as *const c_void);
        }

        unsafe
        {
            let mut texture1 = 0;        
            // load texture atlas
            let img = image::open("rust-vox/textures/atlas.png").unwrap().flipv();
            let width = img.width();
            let height = img.height();
            let data = img.as_bytes();

            gl::GenTextures(1, &mut texture1);
            
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture1);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _ );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _ );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST_MIPMAP_NEAREST as _ );
            gl::TexParameteri( gl::TEXTURE_2D , gl::TEXTURE_MAG_FILTER , gl::NEAREST as _ );

            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as _ , width.try_into().unwrap() , height.try_into().unwrap() ,
                0, gl::RGBA as _ , gl::UNSIGNED_BYTE , data.as_ptr().cast() );
            gl::GenerateMipmap(gl::TEXTURE_2D);

            // ------------------------------------------ END 

            gl::BindVertexArray(0); // unbind VAO
            gl::BindBuffer(gl::ARRAY_BUFFER , 0); // unbind currently bound buffer

            // load the program

            let default_shader = Shader::new("rust-vox/shaders/default_vertex.vert".to_string(),
             "rust-vox/shaders/default_fragment.frag".to_string() ).expect("Shader Error");
            
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
            gl::FrontFace(gl::CW);

            let shadow_shader =  Shader::new("rust-vox/shaders/shadow_vertex.vert".to_string(),
            "rust-vox/shaders/shadow_fragment.frag".to_string() ).expect("Shader Error");

            // generate a framebuffer for the shadow map

            let mut shadow_fb = 0;
            gl::GenFramebuffers(1, &mut shadow_fb);

            // TODO: remove parameters from here
            let shadow_height: i32 = 2048;
            let shadow_width: i32 = 2048;

            let mut depth_texture = 0;
            gl::GenTextures(1, &mut depth_texture);
            gl::BindTexture(gl::TEXTURE_2D, depth_texture);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::DEPTH_COMPONENT as _ , shadow_width, shadow_height,
                0, gl::DEPTH_COMPONENT, gl::FLOAT , 0 as * const c_void );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER,gl::NEAREST as _ );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER,gl::NEAREST as _ );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as _ );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as _ );
            let border_color: [f32;4] = [1.0,1.0,1.0,1.0];
            gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR, border_color.as_ptr() );

            gl::BindFramebuffer(gl::FRAMEBUFFER, shadow_fb);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, depth_texture, 0);
            // need to explicitely mention that we will render no color on this framebuffer
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);
            // unbind
            gl::BindFramebuffer( gl::FRAMEBUFFER, 0);
            
            // TEST, RENDERING THE SUN
            let mut sun = Mesh::new();
            let vox = Voxel::new(super::voxel::VoxelType::Grass,true);
            vox.append_mesh_faces(&[true,true,true,true,true,true],Vec3::new(0.0,100.0,0.0) , &mut sun);
            sun.upload();

            Renderer { default_shader , shadow_shader , texture1 , shadow_fb , depth_texture , sun }
        }
    }

    pub fn draw_world(&mut self, world: &World)
    {   
        unsafe
        {
            // PASS 1: render to the shadow map
            self.shadow_shader.bind();

            gl::Viewport(0, 0, 2048, 2048);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.shadow_fb);
            gl::Clear(gl::DEPTH_BUFFER_BIT);

            let light_projection = Mat4::orthographic_rh_gl(-100.0, 100.0, -100.0, 100.0, 1.0, 500.0);
            let light_look_at = Mat4::look_at_rh(Vec3::new(0.0,100.0,0.0),world.camera.position,Vec3::new(0.0,1.0,0.0));
            let shadow_light_transform = light_projection * light_look_at;
        
            self.shadow_shader.set_uniform_matrix4fv("transform", &shadow_light_transform)
                .expect("error setting the shadow light transform");

            self.draw_geometry(world);
            
            // PASS 2: render the scene normally
            self.default_shader.bind();

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0); // bind default framebuffer
            gl::Viewport(0, 0, 1700 ,900);
            gl::ClearColor(0.25,0.5,0.88,1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture1);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, self.depth_texture);

            // FIXME: setting the constant uniforms at every draw ?
            self.default_shader.set_uniform1i("texture_atlas", 0).expect("error setting the texture uniform");
            self.default_shader.set_uniform1i("shadow_map", 1).expect("error setting the shadow map texture uniform");

            let projection = Mat4::perspective_rh_gl(PI/4.0, 800.0/600.0, 0.1, 1000.0);
            // camera matrix 
            let trans =  projection * world.camera.get_look_at();
            self.default_shader.set_uniform_matrix4fv("transform", &trans).expect("error setting the transform uniform");
            self.default_shader.set_uniform_matrix4fv("light_transform", &shadow_light_transform).expect("error setting the light transform uniform");
        
            self.draw_geometry(world);
            Shader::unbind();
        }   
    }

    fn draw_geometry(&mut self, world: &World)
    {
        // draw each chunk's mesh
        for chunk in world.chunk_manager.get_chunks_to_render().iter()
        {
            let chunk = chunk.borrow_mut();
            if let Some(offset) = chunk.animation.as_ref()
            {
                self.default_shader.set_uniform3fv("animation_offset", &offset.current ).expect("error setting animation offset!");
            }
            else
            {
                self.default_shader.set_uniform3fv("animation_offset", &Vec3::ZERO).expect("error setting animation offset!");
            }

            Renderer::draw_mesh(chunk.mesh.as_ref().expect("mesh was not initialized!"));
        }

        // draw the sun
        self.default_shader.set_uniform3fv("animation_offset", &Vec3::ZERO).expect("error setting animation offset!");
        Renderer::draw_mesh(&self.sun);
    }

    pub fn draw_mesh(mesh: &Mesh)
    {
        unsafe
        {
            mesh.vao.as_ref().unwrap().bind();
            gl::DrawElements(gl::TRIANGLES, mesh.indices.len() as _  , gl::UNSIGNED_INT, 0 as _ );
            VertexArray::unbind();
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