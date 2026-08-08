use eframe::glow::{self, HasContext as _};

const VERT_SRC: &str = r#"
attribute vec2 position;
varying vec2 vUv;

void main() {
    vUv = position * 0.5 + 0.5;
    gl_Position = vec4(position, 0.0, 1.0);
}
"#;

const FRAG_SRC: &str = r#"
precision highp float;
varying vec2 vUv;
uniform float u_time;

vec3 blob_color(vec2 uv, vec2 center, vec3 color, float radius) {
    float d = distance(uv, center);
    float falloff = smoothstep(radius, 0.0, d);
    return color * falloff;
}

float hash(vec2 p) {
    return fract(sin(dot(p, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
    vec2 uv = vUv;
    float t = u_time * 0.06;

    vec2 c1 = vec2(0.30 + 0.25 * sin(t * 1.3), 0.65 + 0.20 * cos(t * 0.9));
    vec2 c2 = vec2(0.75 + 0.20 * cos(t * 1.1), 0.30 + 0.25 * sin(t * 1.6));
    vec2 c3 = vec2(0.50 + 0.30 * sin(t * 0.7 + 2.0), 0.50 + 0.30 * cos(t * 0.5 + 1.0));
    vec2 c4 = vec2(0.20 + 0.22 * cos(t * 0.9 + 4.0), 0.25 + 0.22 * sin(t * 1.2 + 1.5));

    vec3 col = vec3(0.06, 0.06, 0.10);
    col += blob_color(uv, c1, vec3(0.35, 0.35, 0.85), 0.85);
    col += blob_color(uv, c2, vec3(0.85, 0.40, 0.60), 0.85);
    col += blob_color(uv, c3, vec3(0.95, 0.65, 0.35), 0.75);
    col += blob_color(uv, c4, vec3(0.30, 0.55, 0.75), 0.60);

    // Soft vignette: flat corners otherwise read as a cheap, unfinished fill.
    float vign = smoothstep(0.9, 0.25, distance(uv, vec2(0.5)));
    col *= mix(0.78, 1.0, vign);

    // Dither to hide banding on the smooth color ramps.
    float grain = (hash(gl_FragCoord.xy + u_time) - 0.5) * (1.0 / 64.0);
    col += grain;

    gl_FragColor = vec4(col, 1.0);
}
"#;

/// A fullscreen animated gradient, rendered with a hand-rolled GLSL shader
/// via an `egui::PaintCallback` into the glow context eframe already owns.
pub struct GradientBackground {
    program: glow::Program,
    vao: glow::VertexArray,
    u_time: Option<glow::UniformLocation>,
}

impl GradientBackground {
    pub fn new(gl: &glow::Context) -> Self {
        unsafe {
            let vertex = compile(gl, glow::VERTEX_SHADER, VERT_SRC);
            let fragment = compile(gl, glow::FRAGMENT_SHADER, FRAG_SRC);

            let program = gl.create_program().expect("create gradient program");
            gl.attach_shader(program, vertex);
            gl.attach_shader(program, fragment);
            gl.link_program(program);
            assert!(
                gl.get_program_link_status(program),
                "gradient shader link error: {}",
                gl.get_program_info_log(program)
            );
            gl.detach_shader(program, vertex);
            gl.detach_shader(program, fragment);
            gl.delete_shader(vertex);
            gl.delete_shader(fragment);

            let vbo = gl.create_buffer().expect("create gradient vbo");
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            let verts: [f32; 6] = [-1.0, -1.0, 3.0, -1.0, -1.0, 3.0];
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&verts),
                glow::STATIC_DRAW,
            );

            let vao = gl.create_vertex_array().expect("create gradient vao");
            gl.bind_vertex_array(Some(vao));
            let loc = gl
                .get_attrib_location(program, "position")
                .expect("gradient position attribute");
            gl.enable_vertex_attrib_array(loc);
            gl.vertex_attrib_pointer_f32(loc, 2, glow::FLOAT, false, 0, 0);
            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);

            let u_time = gl.get_uniform_location(program, "u_time");

            Self {
                program,
                vao,
                u_time,
            }
        }
    }

    pub fn paint(&self, gl: &glow::Context, time: f32) {
        unsafe {
            gl.use_program(Some(self.program));
            gl.bind_vertex_array(Some(self.vao));
            if let Some(loc) = &self.u_time {
                gl.uniform_1_f32(Some(loc), time);
            }
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
            gl.bind_vertex_array(None);
        }
    }
}

unsafe fn compile(gl: &glow::Context, kind: u32, source: &str) -> glow::Shader {
    unsafe {
        let shader = gl.create_shader(kind).expect("create shader");
        gl.shader_source(shader, source);
        gl.compile_shader(shader);
        assert!(
            gl.get_shader_compile_status(shader),
            "gradient shader compile error: {}",
            gl.get_shader_info_log(shader)
        );
        shader
    }
}
