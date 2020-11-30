use gl32::types::{GLchar, GLfloat, GLint, GLuint, GLvoid};
use raw_window_handle::RawWindowHandle;
use std::mem::size_of;
use surfman::{
    Connection, Context, ContextAttributeFlags, ContextAttributes, Device,
    GLVersion,
};
use surfman::{Surface, SurfaceAccess, SurfaceType};

mod shader;

use shader::{Shader, ShaderProgram};

pub struct Renderer {
    device: Device,
    context: Context,
    shader_program: ShaderProgram,
    vbo: GLuint,
    vao: GLuint,
    gl: gl32::Gl,
}

impl Renderer {
    pub fn new(raw_handle: RawWindowHandle) -> Self {
        let connection = Connection::new().unwrap();
        let native_widget = connection
            .create_native_widget_from_rwh(raw_handle)
            .unwrap();
        let adapter = connection.create_low_power_adapter().unwrap();
        let mut device = connection.create_device(&adapter).unwrap();

        let context_attributes = ContextAttributes {
            version: GLVersion::new(3, 2),
            flags: ContextAttributeFlags::ALPHA,
        };
        let context_descriptor = device
            .create_context_descriptor(&context_attributes)
            .unwrap();

        let surface_type = SurfaceType::Widget { native_widget };
        let mut context =
            device.create_context(&context_descriptor, None).unwrap();
        let surface = device
            .create_surface(&context, SurfaceAccess::GPUOnly, surface_type)
            .unwrap();
        device
            .bind_surface_to_context(&mut context, surface)
            .unwrap();
        device.make_context_current(&context).unwrap();

        let gl = gl32::Gl::load_with(|s| device.get_proc_address(&context, s));

        let shader_program = {
            let vertex_shader = Shader::new(
                include_str!("../shaders/vertex.glsl"),
                gl32::VERTEX_SHADER,
                &gl,
            )
            .unwrap();

            let fragment_shader = Shader::new(
                include_str!("../shaders/fragment.glsl"),
                gl32::FRAGMENT_SHADER,
                &gl,
            )
            .unwrap();

            ShaderProgram::new(vertex_shader, fragment_shader, &gl).unwrap()
        };

        unsafe {
            let mut vao = 0;
            gl.GenVertexArrays(1, &mut vao);
            let mut vbo = 0;
            gl.GenBuffers(1, &mut vbo);

            // Bind the Vertex Array Object first, then bind and set vertex buffer(s).
            gl.BindVertexArray(vao);
            gl.BindBuffer(gl32::ARRAY_BUFFER, vbo);
            gl.BufferData(
                gl32::ARRAY_BUFFER,
                (VERTICES.len() * size_of::<f32>()) as isize,
                VERTICES.as_ptr() as *const GLvoid,
                gl32::STATIC_DRAW,
            );

            // Set up shader attributes.
            shader_program.vertex_attrib(
                "position",
                gl32::FLOAT,
                size_of::<GLfloat>(),
                2,
                &gl,
            );

            // Unbind buffers.
            gl.BindBuffer(gl32::ARRAY_BUFFER, 0);
            gl.BindVertexArray(0);

            Self {
                device,
                context,
                shader_program,
                vbo,
                vao,
                gl,
            }
        }
    }

    pub fn render(&mut self, present: bool) {
        unsafe {
            let fbo = match self.device.context_surface_info(&self.context) {
                Ok(Some(surface_info)) => surface_info.framebuffer_object,
                _ => 0,
            };

            self.gl.BindFramebuffer(gl32::FRAMEBUFFER, fbo);

            self.gl.ClearColor(0.12, 0.12, 0.12, 1.0); // Set background color
            self.gl.Clear(gl32::COLOR_BUFFER_BIT); // Clear the color buffer

            self.gl.UseProgram(self.shader_program.object);
            self.gl.BindVertexArray(self.vao);
            self.gl.DrawArrays(gl32::TRIANGLES, 0, 3);
        }

        if present {
            let mut surface = self
                .device
                .unbind_surface_from_context(&mut self.context)
                .unwrap()
                .unwrap();
            self.device
                .present_surface(&mut self.context, &mut surface)
                .unwrap();
            self.device
                .bind_surface_to_context(&mut self.context, surface)
                .unwrap();
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.shader_program.delete(&self.gl);
        self.device.destroy_context(&mut self.context).unwrap();
    }
}

static VERTICES: [f32; 6] = [0.0, 0.5, 0.5, -0.5, -0.5, -0.5];
