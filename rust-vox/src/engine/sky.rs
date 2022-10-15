// everything related to rendering and animating the clouds and celestial bodies
// including the skybox

// ==== Assumptions ====
// East is defined by the direction vector which is the bisector of the -Z axis and +X axis
// West is defined by the direction vector which is the bisector of the +Z axis and -X axis (opposite direction of east)
// The Sun rises from the east and sets west
// The Moon rises from the east and sets west
// No particular motion or position will be set for stars

pub mod sky_state
{
    const TIME_MULTIPLIER: f32 = 3000.0;

    use std::time::Instant;
    use glam::{Vec3, const_vec3};
    use core::f32::consts::PI;

    const PHASE_CONFIG : [SkyState;4] =
    [
        SkyState{start_time:0.0,moon_present:false,sun_present:true,pos_moon:(PI,PI/8.0),pos_sun:(0.0,PI/8.0),
            sky_box_color: [const_vec3!([1.0, 1.0,1.0]),const_vec3!([0.0, 0.0,0.0]),const_vec3!([0.0, 0.0,0.0]),const_vec3!([1.0, 1.0,1.0]),
                            const_vec3!([1.0, 1.0,1.0]),const_vec3!([1.0, 1.0,1.0]),const_vec3!([1.0, 1.0,1.0]),const_vec3!([1.0, 1.0,1.0])]},
            
        SkyState{start_time:2.0,moon_present:false,sun_present:true,pos_moon:(0.0,PI/8.0),pos_sun:(30.0*PI/180.0,PI/8.0),
            sky_box_color: [const_vec3!([1.0,0.0,0.0]),const_vec3!([1.0,0.0,0.0]),const_vec3!([1.0,0.0,0.0]),const_vec3!([1.0,0.0,0.0]),
            const_vec3!([1.0,0.0,0.0]),const_vec3!([1.0,0.0,0.0]),const_vec3!([1.0,0.0,0.0]),const_vec3!([1.0,0.0,0.0])]},

        SkyState{start_time:10.0,moon_present:false,sun_present:true,pos_moon:(0.0,PI/8.0),pos_sun:(150.0*PI/180.0,PI/8.0),
            sky_box_color: [const_vec3!([0.0,1.0,0.0]),const_vec3!([0.0,1.0,0.0]),const_vec3!([0.0,1.0,0.0]),const_vec3!([0.0,1.0,0.0]),
                            const_vec3!([0.0,1.0,0.0]),const_vec3!([0.0,1.0,0.0]),const_vec3!([0.0,1.0,0.0]),const_vec3!([0.0,1.0,0.0])]},

        SkyState{start_time:12.0,moon_present:true,sun_present:false,pos_moon:(0.0,PI/8.0),pos_sun:(PI,PI/8.0),
            sky_box_color: [const_vec3!([0.0,0.0,1.0]),const_vec3!([0.0,0.0,1.0]),const_vec3!([0.0,0.0,1.0]),const_vec3!([0.0,0.0,1.0]),
            const_vec3!([0.0,0.0,1.0]),const_vec3!([0.0,0.0,1.0]),const_vec3!([0.0,0.0,1.0]),const_vec3!([0.0,0.0,1.0])]},
    ];


    enum DayNightPhase
    {
        SunRise,
        Noon,
        Sunset,
        Night,
    }

    impl TryFrom<i32> for DayNightPhase
    {
        type Error = ();

        fn try_from(value: i32) -> Result<Self, Self::Error>
        {
            match value
            {
                x if x == Self::SunRise as i32 => Ok(Self::SunRise),
                x if x == Self::Noon as i32 => Ok(Self::Noon),
                x if x == Self::Sunset as i32 => Ok(Self::Sunset),
                x if x == Self::Night as i32 => Ok(Self::Night),
                _ => Err(())
            }
        }
    }

    impl DayNightPhase
    {
        fn get_next(&self) -> DayNightPhase
        {
            let x = *self as i32;
            ((x+1)%4).try_into().expect("error converting i32 into enum")
        }
    }

    #[derive(Clone)]
    pub struct SkyState
    {
        pub start_time: f32,
        pub sun_present: bool,
        pub moon_present: bool,
        //TODO: document phi and teta
        pub pos_sun: (f32,f32), // phi and teta
        pub pos_moon: (f32,f32),
        pub sky_box_color: [Vec3;8],
    }

    impl SkyState
    {
        pub fn lerp(&self, dest: &SkyState, s: f32) -> SkyState
        {
            // only positions and colors are lerped
            let pos_sun = (self.pos_sun.0 + (dest.pos_sun.0 - self.pos_sun.0) * s, self.pos_sun.1 + (dest.pos_sun.1 - self.pos_sun.1) * s);
            let pos_moon= (self.pos_moon.0 + (dest.pos_moon.0 - self.pos_moon.0) * s, self.pos_moon.1 + (dest.pos_moon.1 - self.pos_moon.1) * s);
            let mut sky_box_color = [Vec3::ZERO;8];

            (0..8).for_each(|i| {
                sky_box_color[i] = self.sky_box_color[i].lerp(dest.sky_box_color[i], s);
            });

            SkyState { start_time: self.start_time, sun_present: self.sun_present, moon_present: self.moon_present, pos_sun, pos_moon, sky_box_color }
        }

        // get a unit vector pointing to the sun (starting at the origin)
        pub fn get_sun_direction(&self) -> Vec3
        {
            let y_coord = self.pos_sun.0.sin();
            let xz_projection = self.pos_sun.0.cos();
            // project the bisector onto X and Z
            let x_coord = xz_projection * self.pos_sun.1.cos();
            let z_coord = xz_projection * self.pos_sun.1.sin();

            Vec3::new(x_coord,y_coord,z_coord)
        }
    }

    pub struct Sky
    {
        current_phase: DayNightPhase,
        pub current_sky_state: SkyState,

        time: Instant // time of day in seconds
    }

    impl Sky
    {
        pub fn new() -> Self
        {
            let current_phase = DayNightPhase::SunRise;
            let current_sky_state = PHASE_CONFIG[0].clone(); // start off in first phase of list

            let time = Instant::now();
            Self{current_phase,current_sky_state,time}
        }

        pub fn update(&mut self)
        {
            // make sure to reset the timer accordingly
            if self.time.elapsed().as_secs_f32() * TIME_MULTIPLIER >= 24.0*3600.0
            {
                self.time = Instant::now(); // restart timer
            }
    
            let mut current_phase = &PHASE_CONFIG[self.current_phase as usize];
            let next_phase = &PHASE_CONFIG[self.current_phase.get_next() as usize];
            
            let mut denom = next_phase.start_time - current_phase.start_time;
            if denom < 0.0 {denom += 24.0;} // happens only in the case where the clock is wrapping around. ex: current start 13:00h, next phase start 2:00h
    
            let mut nume = self.time.elapsed().as_secs_f32() * TIME_MULTIPLIER - current_phase.start_time * 3600.0;
            if nume < 0.0 {nume += 24.0 * 3600.0;}
    
            let mut progress = nume / (denom * 3600.0);
            
            if progress >= 1.0 // current phase is done, proceed to next phase
            {
                self.current_phase = self.current_phase.get_next();
                current_phase = &PHASE_CONFIG[self.current_phase as usize];
                progress = 0.0;
            }
    
            // interpolate positions and colors between the two phases
            self.current_sky_state = current_phase.lerp(next_phase, progress);
        }

        /// get a unit vector pointing to the sun (starting at the origin)
        pub fn get_sun_direction(&self) -> Vec3
        {
            self.current_sky_state.get_sun_direction()
        }

        /// returns true if the sun is up
        pub fn is_sun_present(&self) -> bool
        {
            self.current_sky_state.sun_present
        }

    }

}

pub mod sky_renderer
{
    use glam::{Vec3, Vec2, Mat4};

    use crate::engine::{geometry::{opengl_vertex::OpenglVertex, mesh::Mesh}, renderer::{opengl_abstractions::{vertex_array::VertexLayout, shader::Shader}, Renderer}, voxel::VoxelVertex};

    use super::sky_state::Sky;

    struct SkyBoxVertex
    {
        position: Vec3, // X, Y, Z
        color: Vec3, // R, G ,B
    }

    impl SkyBoxVertex
    {
        pub fn new(position: Vec3, color: Vec3) -> Self
        {
            Self{position,color}
        }
    }

    impl OpenglVertex for SkyBoxVertex
    {
        fn get_layout() -> VertexLayout
        {
            let mut vertex_layout = VertexLayout::new();

            vertex_layout.push_f32(3); // vertex(x,y,z)
            vertex_layout.push_f32(3); // color(r,g,b)

            vertex_layout
        }
    }
    pub struct SkyRenderer
    {
        // renderer inner state
        celestial_shader: Shader,
        skybox_shader: Shader,
        sky_quad: Mesh<VoxelVertex>,
        sun_quad: Mesh<VoxelVertex>,
        moon_quad: Mesh<VoxelVertex>,
        sky_box: Mesh<SkyBoxVertex>,
        tick: f32,
    }
    
    impl SkyRenderer
    {
        pub fn new() -> Self
        {
            // Initialise everything needed to render the sky + objects
            let celestial_shader = Shader::new_from_vs_fs("rust-vox/shaders/celestial.vert", "rust-vox/shaders/celestial.frag").expect("Shader Error");
            // create sky plane
            let mut sky_quad = Mesh::new();
            //TODO: refactor needed, we should be able to customize the Attributes for according to each Shader
            // define it anti-clockwise, no need to rotate it in this case
            let i1 = sky_quad.add_vertex(VoxelVertex::new(Vec3::new(-1.0,0.2,1.0),0,Vec2::new(0.0,0.0)));
            let i2 = sky_quad.add_vertex(VoxelVertex::new(Vec3::new(-1.0,0.2,-1.0),0,Vec2::new(0.0,1.0)));
            let i3 = sky_quad.add_vertex(VoxelVertex::new(Vec3::new(1.0,0.2,-1.0),0,Vec2::new(1.0,1.0)));
            let i4 = sky_quad.add_vertex(VoxelVertex::new(Vec3::new(1.0,0.2,1.0),0,Vec2::new(1.0,0.0)));
    
            sky_quad.add_triangle(i4, i2, i1);
            sky_quad.add_triangle(i2, i4, i3);
    
            sky_quad.upload();
    
            let mut cloud_texture = 0;        
            // load texture atlas
            let img = image::open("rust-vox/textures/clouds.png").unwrap().flipv();
            let width = img.width();
            let height = img.height();
            let data = img.as_bytes();
    
            unsafe
            {
                gl::GenTextures(1, &mut cloud_texture);
                
                gl::ActiveTexture(gl::TEXTURE2);
                gl::BindTexture(gl::TEXTURE_2D, cloud_texture);
    
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _ );
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _ );
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _ );
                gl::TexParameteri(gl::TEXTURE_2D ,gl::TEXTURE_MAG_FILTER, gl::NEAREST as _ );
    
                gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as _ , width.try_into().unwrap() , height.try_into().unwrap() ,
                    0, gl::RGBA as _ , gl::UNSIGNED_BYTE , data.as_ptr().cast() );
                gl::GenerateMipmap(gl::TEXTURE_2D);
            }
    
            // TESTING
            // generate the background, renderer as a unit cube around the player
            //TODO: migrate this so it uses the same shader, since even if the unit cube is renderer with
            // perspective projection, the corners are not noticable due to the small cube size
            
            let mut sky_box = Mesh::new();
    
            // bottom 4
            let i1 = sky_box.add_vertex(SkyBoxVertex::new(Vec3::new(-1.0,-0.2,1.0),Vec3::ZERO));
            let i2 = sky_box.add_vertex(SkyBoxVertex::new(Vec3::new(-1.0,-0.2,-1.0),Vec3::ZERO));
            let i3 = sky_box.add_vertex(SkyBoxVertex::new(Vec3::new(1.0,-0.2,-1.0),Vec3::ZERO));
            let i4 = sky_box.add_vertex(SkyBoxVertex::new(Vec3::new(1.0,-0.2,1.0),Vec3::ZERO));
    
            // top 4
            let i5 = sky_box.add_vertex(SkyBoxVertex::new(Vec3::new(-1.0,0.2,1.0),Vec3::ZERO));
            let i6 = sky_box.add_vertex(SkyBoxVertex::new(Vec3::new(-1.0,0.2,-1.0),Vec3::ZERO));
            let i7 = sky_box.add_vertex(SkyBoxVertex::new(Vec3::new(1.0,0.2,-1.0),Vec3::ZERO));
            let i8 = sky_box.add_vertex(SkyBoxVertex::new(Vec3::new(1.0,0.2,1.0),Vec3::ZERO));
    
            // we are sitting at the origin looking down -Z
            // define triangles
            // bottom plane
            sky_box.add_quad(i1,i2,i3, i4);
            // top plane
            sky_box.add_quad(i8,i7,i6,i5);
            // left plane
            sky_box.add_quad(i1,i5,i6,i2);
            // right plane
            sky_box.add_quad(i3,i7,i8,i4);
            // front plane
            sky_box.add_quad(i2,i6,i7,i3);
            // back plane
            sky_box.add_quad(i4,i8,i5,i1);
    
            sky_box.upload();
    
            let skybox_shader = Shader::new_from_vs_gs_fs("rust-vox/shaders/skybox.vert",
            "rust-vox/shaders/skybox.geom", "rust-vox/shaders/skybox.frag").expect("Shader Error");
    
            // generate the sun
            // the sun is just a textured quad
            let mut sun_quad = Mesh::new();
            
            let i1 = sun_quad.add_vertex(VoxelVertex::new(Vec3::new(-1.0,-1.0,-5.0),0,Vec2::new(0.0,0.0)));
            let i2 = sun_quad.add_vertex(VoxelVertex::new(Vec3::new(-1.0,1.0,-5.0),0,Vec2::new(0.0,1.0)));
            let i3 = sun_quad.add_vertex(VoxelVertex::new(Vec3::new(1.0,1.0,-5.0),0,Vec2::new(1.0,1.0)));
            let i4 = sun_quad.add_vertex(VoxelVertex::new(Vec3::new(1.0,-1.0,-5.0),0,Vec2::new(1.0,0.0)));
            
            sun_quad.add_triangle(i1, i2, i4);
            sun_quad.add_triangle(i2, i3, i4);
    
            sun_quad.upload();
    
            let mut sun_texture = 0;        
            // load texture atlas
            let img = image::open("rust-vox/textures/sun.png").unwrap().flipv();
            let width = img.width();
            let height = img.height();
            let data = img.as_bytes();
    
            unsafe
            {
                gl::GenTextures(1, &mut sun_texture);
                
                gl::ActiveTexture(gl::TEXTURE3);
                gl::BindTexture(gl::TEXTURE_2D, sun_texture);
    
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _ );
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _ );
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _ );
                gl::TexParameteri(gl::TEXTURE_2D ,gl::TEXTURE_MAG_FILTER, gl::NEAREST as _ );
    
                gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as _ , width.try_into().unwrap() , height.try_into().unwrap() ,
                    0, gl::RGBA as _ , gl::UNSIGNED_BYTE , data.as_ptr().cast() );
                gl::GenerateMipmap(gl::TEXTURE_2D);
            }
    
            // setup the moon texture
            let mut moon_quad = Mesh::new();
            
            let i1 = moon_quad.add_vertex(VoxelVertex::new(Vec3::new(-1.0,-1.0,-5.0),0,Vec2::new(0.0,0.0)));
            let i2 = moon_quad.add_vertex(VoxelVertex::new(Vec3::new(-1.0,1.0,-5.0),0,Vec2::new(0.0,1.0)));
            let i3 = moon_quad.add_vertex(VoxelVertex::new(Vec3::new(1.0,1.0,-5.0),0,Vec2::new(1.0,1.0)));
            let i4 = moon_quad.add_vertex(VoxelVertex::new(Vec3::new(1.0,-1.0,-5.0),0,Vec2::new(1.0,0.0)));
            
            moon_quad.add_triangle(i1, i2, i4);
            moon_quad.add_triangle(i2, i3, i4);
    
            moon_quad.upload();
    
            let mut moon_texture = 0;        
            // load texture atlas
            let img = image::open("rust-vox/textures/moon.png").unwrap().flipv();
            let width = img.width();
            let height = img.height();
            let data = img.as_bytes();
    
            unsafe
            {
                gl::GenTextures(1, &mut moon_texture);
                
                gl::ActiveTexture(gl::TEXTURE4);
                gl::BindTexture(gl::TEXTURE_2D, moon_texture);
    
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _ );
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _ );
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _ );
                gl::TexParameteri(gl::TEXTURE_2D ,gl::TEXTURE_MAG_FILTER, gl::NEAREST as _ );
    
                gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as _ , width.try_into().unwrap() , height.try_into().unwrap() ,
                    0, gl::RGBA as _ , gl::UNSIGNED_BYTE , data.as_ptr().cast() );
                gl::GenerateMipmap(gl::TEXTURE_2D);
            }
    
            Self{tick:0.0, celestial_shader,skybox_shader,sky_box,sky_quad,sun_quad,moon_quad}
        }
    
        pub fn render(&mut self, sky: &Sky)
        {
            let sky_state = &sky.current_sky_state;
    
            // update the sky mesh
            self.sky_box.respecify_vertices(|vertices| {
                for (index, vert) in vertices.iter_mut().enumerate()
                {
                    vert.color = sky_state.sky_box_color[index];
                }
            });
            
            // setting the depth func to always will make every fragment pass the depth test
            // we need this since we set all sky geometry to have a depth = 1 such that it is always drawn behind foreground objects
            // but Z fighting will exist between sky objects themselves, to solve this we set the depth test to always pass and
            // then draw the sky objects from back to front manually ourselves
            unsafe
            {
                gl::DepthFunc(gl::ALWAYS);
            }
    
            self.skybox_shader.bind();
            Renderer::draw_mesh_with_mode(&self.sky_box, gl::LINES_ADJACENCY);
            Shader::unbind();
    
            // PASS 3: draw celestial bodies
            unsafe
            {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA); 
            }
    
            // remove the translation component from the camera's view matrix, since the background
            // must appear the same distance from any camera position
            // this is done by taking the upper 3x3 matrix from original 4x4 view matrix
            self.celestial_shader.bind();
            self.tick += 0.0001;
            if self.tick > 1.0 {self.tick = 0.0;}
            self.celestial_shader.set_uniform_1f("sub", self.tick).expect("error setting sub float uniform");
            self.celestial_shader.set_uniform1i("text", 2).expect("error setting the sky texture");
            self.celestial_shader.set_uniform_matrix4fv("model", &Mat4::IDENTITY).expect("error setting the view uniform");
            // Renderer::draw_mesh(&self.sky_quad);
    
            if sky_state.sun_present
            {
                self.celestial_shader.set_uniform_1f("sub", 0.0).expect("error setting sub float uniform");
                self.celestial_shader.set_uniform1i("text", 3).expect("error setting the sun texture");
                let sun_quad_trans = Mat4::from_rotation_x(sky_state.pos_sun.0) * Mat4::from_rotation_y(sky_state.pos_sun.1);
                self.celestial_shader.set_uniform_matrix4fv("model", &sun_quad_trans).expect("error setting the model transformation for the sun_quad");
                Renderer::draw_mesh(&self.sun_quad);
            }
    
            if sky_state.moon_present
            {
                self.celestial_shader.set_uniform_1f("sub", 0.0).expect("error setting sub float uniform");
                self.celestial_shader.set_uniform1i("text", 4).expect("error setting the sun texture");
                let moon_quad_trans = Mat4::from_rotation_x(sky_state.pos_moon.0) * Mat4::from_rotation_y(sky_state.pos_moon.1);
                self.celestial_shader.set_uniform_matrix4fv("model", &moon_quad_trans).expect("error setting the model transformation for the sun_quad");
                Renderer::draw_mesh(&self.moon_quad);
            }
    
            unsafe
            {
                gl::Disable(gl::BLEND);
                gl::DepthFunc(gl::LESS);
            }
        }
    }
}

