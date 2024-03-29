// everything related to rendering and animating the clouds and celestial bodies
// including the skybox

// ==== Assumptions ====
// East is defined by the direction vector which is the bisector of the -Z axis and +X axis
// West is defined by the direction vector which is the bisector of the +Z axis and -X axis (opposite direction of east)
// The Sun rises from the east and sets west
// The Moon rises from the east and sets west
// No particular motion or position will be set for stars

const MAX_SECONDS_DAY : f32 = 24.0 * 3600.0 ; // 24 hours * 3600 second per hour

pub mod sky_state
{
    const TIME_MULTIPLIER: f32 = 500.0;

    use std::{time::Instant, fmt::{Display, Formatter}};
    use glam::{Vec3};
    use core::{f32::consts::PI, fmt};

    use super::MAX_SECONDS_DAY;

    const PHASE_CONFIG : [SkyState;4] =
    [
        SkyState{start_time:0.0,moon_present:false,sun_present:true,pos_moon:(PI,PI/8.0),pos_sun:(0.0,PI/8.0),
            sky_box_color: [Vec3::new(1.0, 1.0,1.0),Vec3::new(0.0, 0.0,0.0),Vec3::new(0.0, 0.0,0.0),Vec3::new(1.0, 1.0,1.0),
                            Vec3::new(1.0, 1.0,1.0),Vec3::new(1.0, 1.0,1.0),Vec3::new(1.0, 1.0,1.0),Vec3::new(1.0, 1.0,1.0)]},
            
        SkyState{start_time:2.0,moon_present:false,sun_present:true,pos_moon:(0.0,PI/8.0),pos_sun:(30.0*PI/180.0,PI/8.0),
            sky_box_color: [Vec3::new(1.0,0.0,0.0),Vec3::new(1.0,0.0,0.0),Vec3::new(1.0,0.0,0.0),Vec3::new(1.0,0.0,0.0),
            Vec3::new(1.0,0.0,0.0),Vec3::new(1.0,0.0,0.0),Vec3::new(1.0,0.0,0.0),Vec3::new(1.0,0.0,0.0)]},

        SkyState{start_time:10.0,moon_present:false,sun_present:true,pos_moon:(0.0,PI/8.0),pos_sun:(150.0*PI/180.0,PI/8.0),
            sky_box_color: [Vec3::new(0.0,1.0,0.0),Vec3::new(0.0,1.0,0.0),Vec3::new(0.0,1.0,0.0),Vec3::new(0.0,1.0,0.0),
                            Vec3::new(0.0,1.0,0.0),Vec3::new(0.0,1.0,0.0),Vec3::new(0.0,1.0,0.0),Vec3::new(0.0,1.0,0.0)]},

        SkyState{start_time:12.0,moon_present:true,sun_present:false,pos_moon:(0.0,PI/8.0),pos_sun:(PI,PI/8.0),
            sky_box_color: [Vec3::new(0.0,0.0,1.0),Vec3::new(0.0,0.0,1.0),Vec3::new(0.0,0.0,1.0),Vec3::new(0.0,0.0,1.0),
            Vec3::new(0.0,0.0,1.0),Vec3::new(0.0,0.0,1.0),Vec3::new(0.0,0.0,1.0),Vec3::new(0.0,0.0,1.0)]},
    ];

    #[derive(Copy,Clone)]
    pub enum DayNightPhase
    {
        SunRise,
        Noon,
        Sunset,
        Night,
    }

    impl Display for DayNightPhase {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
           match *self {
               DayNightPhase::SunRise => write!(f, "SunRise"),
               DayNightPhase::Noon => write!(f, "Noon"),
               DayNightPhase::Sunset => write!(f, "Sunset"),
               DayNightPhase::Night => write!(f, "Night"),
           }
        }
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

        /// get a unit vector pointing to the sun (starting at the origin)
        // we are assuming that teta is defined between the X axis and the -Z axis
        pub fn get_sun_direction(&self) -> Vec3
        {
            let y_coord = self.pos_sun.0.sin().abs();
            let xz_projection = self.pos_sun.0.cos().abs();
            // project the bisector onto X and Z
            let x_coord = xz_projection * self.pos_sun.1.cos();
            let z_coord = -xz_projection * self.pos_sun.1.sin();

            Vec3::new(x_coord,y_coord,z_coord).normalize()
        }
    }

    pub struct Sky
    {
        current_phase: DayNightPhase,
        pub current_sky_state: SkyState,

        timer: Instant, // time of day in seconds
        time: f32, // used as the current time if the timer is not running
        is_halted: bool,
    }

    impl Default for Sky
    {
        fn default() -> Self
        {
            let current_phase = DayNightPhase::SunRise;
            let current_sky_state = PHASE_CONFIG[0].clone(); // start off in first phase of list

            let timer = Instant::now();
            Self{current_phase,current_sky_state,timer, time: 5.0 * 3600.0 , is_halted: true}
        }
    }

    impl Sky
    {
        pub fn update(&mut self)
        {
            if !self.is_halted
            {
                self.time += self.timer.elapsed().as_secs_f32() * TIME_MULTIPLIER;
                self.timer = Instant::now(); // restart timer between function calls
            }

            // make sure to reset the timer accordingly
            if self.time >= MAX_SECONDS_DAY
            {
                self.time = 0.0;
            }
    
            let mut current_phase = &PHASE_CONFIG[self.current_phase as usize];
            let next_phase = &PHASE_CONFIG[self.current_phase.get_next() as usize];
            
            let mut denom = next_phase.start_time - current_phase.start_time;
            if denom < 0.0 {denom += 24.0;} // happens only in the case where the clock is wrapping around. ex: current start 13:00h, next phase start 2:00h
    
            let mut nume = self.time - current_phase.start_time * 3600.0;
            if nume < 0.0 {nume += MAX_SECONDS_DAY;}
    
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

        /// Dictates whether the Day-Night Sky cycle is running
        pub fn set_halted(&mut self, is_halted: bool)
        {
            self.is_halted = is_halted;

            if !self.is_halted
            {
                self.timer = Instant::now(); // reset the timer value on re-enable
            }
        }

        pub fn is_halted(&self) -> bool
        {
            self.is_halted
        }

        /// Sets the current time in the Day-Night Sky cycle
        pub fn set_time_hours(&mut self, time_hours: f32)
        {
            if time_hours.is_sign_negative()
            {
                return;
            }

            self.time = time_hours * 3600.0;

            // run one update
            self.update();
        }

        pub fn get_time_secs(&self) -> f32
        {
            self.time
        }

        pub fn get_time_hours(&self) -> f32
        {
            self.time / 3600.0
        }

        pub fn curent_cycle_phase(&self) -> DayNightPhase
        {
            self.current_phase
        }

    }

}

pub mod sky_renderer
{
    use std::{mem};

    use glam::{Vec3, Vec2, Mat4};
    use crate::engine::{geometry::{opengl_vertex::OpenglVertex, mesh::Mesh}, renderer::{opengl_abstractions::{vertex_array::{VertexLayout, VertexArray}, shader::Shader}, Renderer, allocators::{vertex_pool_allocator::Daic, default_allocator::DefaultAllocator}}};
    use super::sky_state::Sky;

    struct SkyBoxVertex
    {
        _position: Vec3, // X, Y, Z
        color: Vec3, // R, G ,B
    }

    impl SkyBoxVertex
    {
        pub fn new(position: Vec3, color: Vec3) -> Self
        {
            Self{_position: position,color}
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

    // used to render things like the sun,moon and the clouds
    struct SkyQuadVertex
    {
        _position: Vec3, // X, Y, Z
        _uv: Vec2,
    }

    impl SkyQuadVertex
    {
        pub fn new(position: Vec3, uv: Vec2) -> Self
        {
            Self{_position: position, _uv: uv}
        }
    }

    impl OpenglVertex for SkyQuadVertex
    {
        fn get_layout() -> VertexLayout
        {
            let mut vertex_layout = VertexLayout::new();

            vertex_layout.push_f32(3); // vertex(x,y,z)
            vertex_layout.push_f32(2); // u,v

            vertex_layout
        }
    }

    pub struct SkyRenderer
    {
        // renderer inner state
        celestial_shader: Shader,
        skybox_shader: Shader,
        sky_quad: Mesh<SkyQuadVertex>,
        sun_quad: Mesh<SkyQuadVertex>,
        moon_quad: Mesh<SkyQuadVertex>,
        sky_box: Mesh<SkyBoxVertex>,
        tick: f32,

        sky_quad_allocator: DefaultAllocator<SkyQuadVertex>,
        sky_box_allocator: DefaultAllocator<SkyBoxVertex>,
        indbo: u32,
    }

    impl Default for SkyRenderer
    {
        fn default() -> Self
        {
            // create the allocators
            let mut sky_quad_allocator = DefaultAllocator::new();
            let mut sky_box_allocator = DefaultAllocator::new();

            // Initialise everything needed to render the sky + objects
            let celestial_shader = Shader::new_from_vs_fs("rust-vox/shaders/celestial.vert", "rust-vox/shaders/celestial.frag").expect("Shader Error");
            // create sky plane
            let mut sky_quad = Mesh::default();
            //TODO: refactor needed, we should be able to customize the Attributes for according to each Shader
            // define it anti-clockwise, no need to rotate it in this case
            let i1 = sky_quad.add_vertex(SkyQuadVertex::new(Vec3::new(-1.0,0.2,1.0),Vec2::new(0.0,0.0)));
            let i2 = sky_quad.add_vertex(SkyQuadVertex::new(Vec3::new(-1.0,0.2,-1.0),Vec2::new(0.0,1.0)));
            let i3 = sky_quad.add_vertex(SkyQuadVertex::new(Vec3::new(1.0,0.2,-1.0),Vec2::new(1.0,1.0)));
            let i4 = sky_quad.add_vertex(SkyQuadVertex::new(Vec3::new(1.0,0.2,1.0),Vec2::new(1.0,0.0)));
    
            sky_quad.add_triangle_indices(i4, i2, i1);
            sky_quad.add_triangle_indices(i2, i4, i3);
    
            sky_quad_allocator.alloc(&mut sky_quad);
    
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
    
            // generate the background, renderer as a unit cube around the player        
            let mut sky_box = Mesh::default();
    
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
            sky_box.add_quad_indices(i1,i2,i3, i4);
            // top plane
            sky_box.add_quad_indices(i8,i7,i6,i5);
            // left plane
            sky_box.add_quad_indices(i1,i5,i6,i2);
            // right plane
            sky_box.add_quad_indices(i3,i7,i8,i4);
            // front plane
            sky_box.add_quad_indices(i2,i6,i7,i3);
            // back plane
            sky_box.add_quad_indices(i4,i8,i5,i1);
    
            sky_box_allocator.alloc(&mut sky_box);
    
            let skybox_shader = Shader::new_from_vs_gs_fs("rust-vox/shaders/skybox.vert",
            "rust-vox/shaders/skybox.geom", "rust-vox/shaders/skybox.frag").expect("Shader Error");
    
            // generate the sun
            // the sun is just a textured quad
            let mut sun_quad = Mesh::default();

            // sun_quad.add_quad(
            //     SkyQuadVertex::new(Vec3::new(-1.0,-1.0,-5.0),Vec2::new(0.0,0.0)),
            //     SkyQuadVertex::new(Vec3::new(-1.0,1.0,-5.0),Vec2::new(0.0,1.0)),
            //     SkyQuadVertex::new(Vec3::new(1.0,1.0,-5.0),Vec2::new(1.0,1.0)),
            //     SkyQuadVertex::new(Vec3::new(1.0,-1.0,-5.0),Vec2::new(1.0,0.0))
            //     );

            sun_quad.add_quad(
                SkyQuadVertex::new(Vec3::new(5.0,-1.0,-1.0),Vec2::new(0.0,0.0)),
                SkyQuadVertex::new(Vec3::new(5.0,1.0,-1.0),Vec2::new(0.0,1.0)),
                SkyQuadVertex::new(Vec3::new(5.0,1.0,1.0),Vec2::new(1.0,1.0)),
                SkyQuadVertex::new(Vec3::new(5.0,-1.0,1.0),Vec2::new(1.0,0.0))
                );

            sky_quad_allocator.alloc(&mut sun_quad);
    
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
            let mut moon_quad = Mesh::default();

            moon_quad.add_quad(
                SkyQuadVertex::new(Vec3::new(-1.0,-1.0,-5.0),Vec2::new(0.0,0.0)),
                SkyQuadVertex::new(Vec3::new(-1.0,1.0,-5.0),Vec2::new(0.0,1.0)),
                SkyQuadVertex::new(Vec3::new(1.0,1.0,-5.0),Vec2::new(1.0,1.0)),
                SkyQuadVertex::new(Vec3::new(1.0,-1.0,-5.0),Vec2::new(1.0,0.0)));
    
            sky_quad_allocator.alloc(&mut moon_quad);
    
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

            // TESTING 
            let commands: [Daic;1] = [Daic::new(sky_quad.indices.len() as u32,1, 0, 0)];
            let mut indbo = 0;

            unsafe
            {
                gl::GenBuffers(1, &mut indbo);
                gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, indbo);

                gl::BufferData(gl::DRAW_INDIRECT_BUFFER, mem::size_of::<Daic>() as isize, commands.as_ptr() as _, gl::STATIC_DRAW);
            }

            // END TEST
    
            Self{sky_box_allocator, sky_quad_allocator ,tick:0.0, celestial_shader,skybox_shader,sky_box,sky_quad,sun_quad,moon_quad, indbo}
        }
    }
    
    impl SkyRenderer
    {
        
        pub fn render(&mut self, sky: &Sky)
        {
            let sky_state = &sky.current_sky_state;
    
            // update the sky mesh
            // self.sky_box.respecify_vertices(|vertices| {
            //     for (index, vert) in vertices.iter_mut().enumerate()
            //     {
            //         vert.color = sky_state.sky_box_color[index];
            //     }
            // });
            
            // setting the depth func to always will make every fragment pass the depth test
            // we need this since we set all sky geometry to have a depth = 1 such that it is always drawn behind foreground objects
            // but Z fighting will exist between sky objects themselves, to solve this we set the depth test to always pass and
            // then draw the sky objects from back to front manually ourselves
            unsafe
            {
                gl::DepthFunc(gl::ALWAYS);
            }
    
            self.skybox_shader.bind();
            Renderer::draw_mesh_with_mode(&self.sky_box_allocator, &self.sky_box, gl::LINES_ADJACENCY);
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


            // // testing, draw the sky_quad
            // unsafe
            // {
            //     self.sky_quad.vao.as_ref().unwrap().bind();
            //     gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, self.indbo);
            //     gl::MultiDrawElementsIndirect(gl::TRIANGLES, gl::UNSIGNED_INT, 0 as _, 1, mem::size_of::<Daic>() as i32);
            //     VertexArray::<SkyQuadVertex>::unbind();
            // }
            // Renderer::draw_mesh(&self.sky_quad);
            
            if sky_state.sun_present
            {
                self.celestial_shader.set_uniform_1f("sub", 0.0).expect("error setting sub float uniform");
                self.celestial_shader.set_uniform1i("text", 3).expect("error setting the sun texture");
                let sun_quad_trans = Mat4::from_rotation_z(sky_state.pos_sun.0) * Mat4::from_rotation_y(sky_state.pos_sun.1);
                self.celestial_shader.set_uniform_matrix4fv("model", &sun_quad_trans).expect("error setting the model transformation for the sun_quad");
                Renderer::draw_mesh(&self.sky_quad_allocator, &self.sun_quad);
            }
    
            if sky_state.moon_present
            {
                self.celestial_shader.set_uniform_1f("sub", 0.0).expect("error setting sub float uniform");
                self.celestial_shader.set_uniform1i("text", 4).expect("error setting the sun texture");
                let moon_quad_trans = Mat4::from_rotation_z(sky_state.pos_moon.0) * Mat4::from_rotation_y(sky_state.pos_moon.1);
                self.celestial_shader.set_uniform_matrix4fv("model", &moon_quad_trans).expect("error setting the model transformation for the sun_quad");
                Renderer::draw_mesh(&self.sky_quad_allocator, &self.moon_quad);
            }
    
            unsafe
            {
                gl::Disable(gl::BLEND);
                gl::DepthFunc(gl::LESS);
            }
        }
    }
}

