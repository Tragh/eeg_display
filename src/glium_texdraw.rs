use glium;

//use conrod::backend::glium::glium;
//use conrod::backend::glium::glium::{DisplayBuild, Surface};
use glium::{Surface};

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);

pub struct GliumTexDraw{
    pub vertex_buffer: glium::VertexBuffer<Vertex>,
    pub indices: glium::index::NoIndices,
    pub program: glium::Program
}


impl GliumTexDraw{
    pub fn new(display: &glium::backend::glutin_backend::GlutinFacade) -> GliumTexDraw{
        let mut square=Vec::<Vertex>::new();
        square.push(Vertex { position: [ -0.5,  -0.5], tex_coords: [0.0, 0.0] });
        square.push(Vertex { position: [ 0.5,  -0.5], tex_coords: [1.0, 0.0] });
        square.push(Vertex { position: [ -0.5,  0.5], tex_coords: [0.0, 1.0] });
        square.push(Vertex { position: [ 0.5,  -0.5], tex_coords: [1.0, 0.0] });
        square.push(Vertex { position: [ 0.5,  0.5], tex_coords: [1.0, 1.0] });
        square.push(Vertex { position: [ -0.5,  0.5], tex_coords: [0.0, 1.0] });


        let vertex_buffer = glium::VertexBuffer::new(display, &square).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        let vertex_shader_src = r#"
            #version 140
            in vec2 position;
            in vec2 tex_coords;
            out vec2 v_tex_coords;
            uniform mat4 matrix;

            void main() {
                v_tex_coords = tex_coords;
                gl_Position = matrix * vec4(position, 0.0, 1.0);
            }
        "#;

        let fragment_shader_src = r#"
            #version 140
            in vec2 v_tex_coords;
            out vec3 color;
            uniform sampler2D tex;
            void main() {
                color = texture(tex, v_tex_coords).rgb;
            }
        "#;

        let program = glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap();
        GliumTexDraw{
            vertex_buffer: vertex_buffer,
            indices: indices,
            program: program
        }
    }

    pub fn draw(&self, target: &mut glium::Frame, texture: &glium::Texture2d, x: f64, y: f64, width: f64, height: f64){
        let uniforms = uniform! {
            matrix: [
            [width as f32, 0.0, 0.0, 0.0],
            [0.0, height as f32, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [x as f32, y as f32, 0.0, 1.0f32],
        ],
            tex: texture.sampled()
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest),
        };
        target.draw(&self.vertex_buffer, &self.indices, &self.program, &uniforms,
                    &Default::default()).unwrap();
    }
}
