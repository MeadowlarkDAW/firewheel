use gl32::types::{GLchar, GLfloat, GLint, GLuint, GLvoid};
use raw_window_handle::RawWindowHandle;
use std::mem::size_of;
use surfman::{
    Connection, Context, ContextAttributeFlags, ContextAttributes, Device,
    GLVersion,
};
use surfman::{Surface, SurfaceAccess, SurfaceType};

struct Buffer {
    pub object: GLuint,
}

impl Buffer {
    pub fn from_data(gl: &gl32::Gl, data: &[u8]) -> Buffer {
        unsafe {
            let mut buffer = 0;
            gl.GenBuffers(1, &mut buffer);
            gl.BindBuffer(gl32::ARRAY_BUFFER, buffer);
            gl.BufferData(
                gl32::ARRAY_BUFFER,
                data.len() as isize,
                data.as_ptr() as *const GLvoid,
                gl32::STATIC_DRAW,
            );
            Buffer { object: buffer }
        }
    }
}

pub struct Renderer {
    device: Device,
    context: Context,
    shader_program: GLuint,
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

        unsafe {
            // compile vertex shader
            let vertex_shader = gl.CreateShader(gl32::VERTEX_SHADER);
            gl.ShaderSource(
                vertex_shader,
                1,
                &(VERTEX_SHADER.as_bytes().as_ptr() as *const GLchar),
                &(VERTEX_SHADER.len() as GLint),
            );
            gl.CompileShader(vertex_shader);
            let mut compile_status = 0;
            gl.GetShaderiv(
                vertex_shader,
                gl32::COMPILE_STATUS,
                &mut compile_status,
            );
            if compile_status != gl32::TRUE as GLint {
                let mut info_log_length = 0;
                gl.GetShaderiv(
                    vertex_shader,
                    gl32::INFO_LOG_LENGTH,
                    &mut info_log_length,
                );
                let mut info_log = vec![0; info_log_length as usize + 1];
                gl.GetShaderInfoLog(
                    vertex_shader,
                    info_log_length,
                    std::ptr::null_mut(),
                    info_log.as_mut_ptr() as *mut _,
                );
                gl.DeleteShader(vertex_shader);
                eprintln!(
                    "Failed to compile shader:\n{}",
                    String::from_utf8_lossy(&info_log)
                );
                panic!("Shader compilation failed!");
            }

            // compile fragment shader
            let fragment_shader = gl.CreateShader(gl32::FRAGMENT_SHADER);
            gl.ShaderSource(
                fragment_shader,
                1,
                &(FRAGMENT_SHADER.as_ptr() as *const GLchar),
                &(FRAGMENT_SHADER.len() as GLint),
            );
            gl.CompileShader(fragment_shader);
            let mut compile_status = 0;
            gl.GetShaderiv(
                fragment_shader,
                gl32::COMPILE_STATUS,
                &mut compile_status,
            );
            if compile_status != gl32::TRUE as GLint {
                let mut info_log_length = 0;
                gl.GetShaderiv(
                    fragment_shader,
                    gl32::INFO_LOG_LENGTH,
                    &mut info_log_length,
                );
                let mut info_log = vec![0; info_log_length as usize + 1];
                gl.GetShaderInfoLog(
                    fragment_shader,
                    info_log_length,
                    std::ptr::null_mut(),
                    info_log.as_mut_ptr() as *mut _,
                );
                gl.DeleteShader(fragment_shader);
                eprintln!(
                    "Failed to compile shader:\n{}",
                    String::from_utf8_lossy(&info_log)
                );
                panic!("Shader compilation failed!");
            }

            // link shaders
            let shader_program = gl.CreateProgram();
            gl.AttachShader(shader_program, vertex_shader);
            gl.AttachShader(shader_program, fragment_shader);
            gl.LinkProgram(shader_program);
            let mut status = 0;
            gl.GetProgramiv(shader_program, gl32::LINK_STATUS, &mut status);
            if status != gl32::TRUE as GLint {
                let mut info_log_length = 0;
                gl.GetProgramiv(
                    shader_program,
                    gl32::INFO_LOG_LENGTH,
                    &mut info_log_length,
                );
                let mut info_log = vec![0; info_log_length as usize + 1];
                gl.GetProgramInfoLog(
                    shader_program,
                    info_log_length,
                    std::ptr::null_mut(),
                    info_log.as_mut_ptr() as *mut _,
                );
                gl.DeleteProgram(shader_program);
                eprintln!(
                    "Failed to create shader program:\n{}",
                    String::from_utf8_lossy(&info_log)
                );
                panic!("Shader program creation failed!");
            }
            gl.DeleteShader(vertex_shader);
            gl.DeleteShader(fragment_shader);

            let mut vao = 0;
            gl.GenVertexArrays(1, &mut vao);
            let mut vbo = 0;
            gl.GenBuffers(1, &mut vbo);

            // Bind the Vertex Array Object first, then bind and set vertex buffer(s), and then configure vertex attributes(s).
            gl.BindVertexArray(vao);

            gl.BindBuffer(gl32::ARRAY_BUFFER, vbo);
            gl.BufferData(
                gl32::ARRAY_BUFFER,
                (VERTICES.len() * size_of::<f32>()) as isize,
                VERTICES.as_ptr() as *const GLvoid,
                gl32::STATIC_DRAW,
            );

            let position_attrib = gl.GetAttribLocation(
                shader_program,
                "position".as_ptr() as *const GLchar,
            );
            gl.VertexAttribPointer(
                position_attrib as u32,
                2,
                gl32::FLOAT,
                gl32::FALSE,
                (2 * size_of::<GLfloat>()) as GLint,
                std::ptr::null(),
            );
            gl.EnableVertexAttribArray(position_attrib as u32);

            gl.BindBuffer(gl32::ARRAY_BUFFER, 0);
            gl.BindVertexArray(0);

            //gl.BindFragDataLocation(shader_program, 0, "outColor".as_ptr() as *const GLchar);

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

            self.gl.UseProgram(self.shader_program);
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
        self.device.destroy_context(&mut self.context).unwrap();
    }
}

static VERTICES: [f32; 6] = [0.0, 0.5, 0.5, -0.5, -0.5, -0.5];

static VERTEX_SHADER: &'static str = include_str!("../shaders/vertex.glsl");
static FRAGMENT_SHADER: &'static str = include_str!("../shaders/fragment.glsl");
