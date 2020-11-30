use gl32::types::{GLchar, GLenum, GLint, GLuint};
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum ShaderError {
    CompilationFailed(String),
    LinkFailed(String),
}

impl Error for ShaderError {}

impl fmt::Display for ShaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShaderError::CompilationFailed(msg) => {
                write!(f, "Failed to compile shader:\n{}", msg)
            }
            ShaderError::LinkFailed(msg) => {
                write!(f, "Failed to link shader program:\n{}", msg)
            }
        }
    }
}

pub struct Shader<'a> {
    pub object: GLuint,
    gl: &'a gl32::Gl,
}

impl<'a> Shader<'a> {
    pub fn new(
        source: &str,
        shader_type: GLenum,
        gl: &'a gl32::Gl,
    ) -> Result<Self, ShaderError> {
        unsafe {
            // Allocate a new shader.
            let shader = gl.CreateShader(shader_type);

            // Set the source code of the shader.
            gl.ShaderSource(
                shader,
                1,
                &(source.as_ptr() as *const GLchar),
                &(source.len() as GLint),
            );

            // Compile the shader code.
            gl.CompileShader(shader);

            // Check for compilation errors.
            let mut status = 0;
            gl.GetShaderiv(shader, gl32::COMPILE_STATUS, &mut status);
            if status != gl32::TRUE as GLint {
                // Get the length of the error message string.
                let mut info_log_length = 0;
                gl.GetShaderiv(
                    shader,
                    gl32::INFO_LOG_LENGTH,
                    &mut info_log_length,
                );

                // Write the error message to a buffer.
                let mut info_log = vec![0; info_log_length as usize + 1];
                gl.GetShaderInfoLog(
                    shader,
                    info_log_length,
                    std::ptr::null_mut(),
                    info_log.as_mut_ptr() as *mut _,
                );
                let error_msg = String::from_utf8_lossy(&info_log);

                // Clear the allocated shader.
                gl.DeleteShader(shader);

                return Err(ShaderError::CompilationFailed(
                    error_msg.to_string(),
                ));
            }

            Ok(Self { object: shader, gl })
        }
    }
}

impl<'a> Drop for Shader<'a> {
    fn drop(&mut self) {
        // Clear the allocated shader.
        unsafe {
            self.gl.DeleteShader(self.object);
        }
    }
}

pub struct ShaderProgram {
    pub object: GLuint,
}

impl ShaderProgram {
    pub fn new(
        vertex_shader: Shader<'_>,
        fragment_shader: Shader<'_>,
        gl: &gl32::Gl,
    ) -> Result<Self, ShaderError> {
        unsafe {
            // Allocate a new shader program.
            let shader_program = gl.CreateProgram();

            // Attach the shaders.
            gl.AttachShader(shader_program, vertex_shader.object);
            gl.AttachShader(shader_program, fragment_shader.object);
            gl.LinkProgram(shader_program);

            // Check for linking errors.
            let mut status = 0;
            gl.GetProgramiv(shader_program, gl32::LINK_STATUS, &mut status);
            if status != gl32::TRUE as GLint {
                // Get the length of the error message string.
                let mut info_log_length = 0;
                gl.GetProgramiv(
                    shader_program,
                    gl32::INFO_LOG_LENGTH,
                    &mut info_log_length,
                );

                // Write the error message to a buffer.
                let mut info_log = vec![0; info_log_length as usize + 1];
                gl.GetProgramInfoLog(
                    shader_program,
                    info_log_length,
                    std::ptr::null_mut(),
                    info_log.as_mut_ptr() as *mut _,
                );
                let error_msg = String::from_utf8_lossy(&info_log);

                // Clear the allocated program.
                gl.DeleteProgram(shader_program);

                return Err(ShaderError::LinkFailed(error_msg.to_string()));
            }

            Ok(Self {
                object: shader_program,
            })
        }
    }

    pub unsafe fn vertex_attrib(
        &self,
        name: &str,
        data_type: GLenum,
        data_type_size: usize,
        len: u16,
        gl: &gl32::Gl,
    ) {
        let attrib =
            gl.GetAttribLocation(self.object, name.as_ptr() as *const GLchar);

        gl.VertexAttribPointer(
            attrib as GLuint,
            len as GLint,
            data_type,
            gl32::FALSE,
            (len as usize * data_type_size) as GLint,
            std::ptr::null(),
        );

        gl.EnableVertexAttribArray(attrib as GLuint);
    }

    // Clear the allocated shader program.
    pub fn delete(&self, gl: &gl32::Gl) {
        unsafe {
            gl.DeleteProgram(self.object);
        }
    }
}
