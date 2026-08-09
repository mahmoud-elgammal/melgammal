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

float hash(vec2 p) {
    return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    float a = hash(i);
    float b = hash(i + vec2(1.0, 0.0));
    float c = hash(i + vec2(0.0, 1.0));
    float d = hash(i + vec2(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

float fbm(vec2 p) {
    float v = 0.0;
    float amp = 0.5;
    for (int i = 0; i < 4; i++) {
        v += amp * noise(p);
        p *= 2.0;
        amp *= 0.5;
    }
    return v;
}

// A gentle rolling ridge line, evoking a grassy hill against open sky —
// an original procedural landscape, not a reproduction of any specific
// photograph.
float hillHeight(float x) {
    float h = 0.30;
    h += 0.05 * sin(x * 3.1 + 0.4);
    h += 0.025 * sin(x * 7.0 + 1.7);
    h -= 0.07 * smoothstep(0.15, 0.55, x) * smoothstep(1.0, 0.55, x);
    return h;
}

void main() {
    vec2 uv = vUv;
    float hillY = hillHeight(uv.x);

    vec3 color;

    if (uv.y > hillY) {
        // Sky: deep blue overhead fading to a soft pale blue near the horizon.
        float skyT = smoothstep(hillY, 1.0, uv.y);
        vec3 skyHorizon = vec3(0.74, 0.87, 0.97);
        vec3 skyTop = vec3(0.18, 0.45, 0.86);
        vec3 sky = mix(skyHorizon, skyTop, skyT);

        // Slowly drifting cloud puffs.
        vec2 cloudUv = vec2(uv.x + u_time * 0.004, uv.y * 1.6);
        float clouds = fbm(cloudUv * 3.0);
        clouds = smoothstep(0.55, 0.78, clouds);
        clouds *= smoothstep(hillY, hillY + 0.35, uv.y);
        sky = mix(sky, vec3(1.0), clouds * 0.85);

        color = sky;
    } else {
        // Grassy hill: brighter near the ridge, deepening toward the bottom.
        float depth = 1.0 - clamp(uv.y / max(hillY, 0.0001), 0.0, 1.0);
        vec3 grassLight = vec3(0.47, 0.74, 0.27);
        vec3 grassDark = vec3(0.11, 0.30, 0.10);
        vec3 grass = mix(grassLight, grassDark, depth);

        float texture = fbm(uv * vec2(6.0, 18.0));
        grass += (texture - 0.5) * 0.05;

        float ridgeGlow = smoothstep(hillY - 0.02, hillY, uv.y);
        grass = mix(grass, vec3(0.78, 0.90, 0.48), ridgeGlow * 0.35);

        color = grass;
    }

    // Dither to hide banding on the smooth gradients.
    float grain = (hash(gl_FragCoord.xy + u_time) - 0.5) * (1.0 / 96.0);
    color += grain;

    gl_FragColor = vec4(color, 1.0);
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
