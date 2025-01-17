use vulkano::device::Device;
use vulkano::shader::{ShaderModule, ShaderModuleCreateInfo };
use std::sync::Arc;
use shaderc::{Compiler, CompileOptions, ShaderKind};
use std::path::Path;
use std::fs::File;
use std::io::Read;

pub struct Shaders<'a> {
    pub vertex: Option<Arc<ShaderModule>>,
    pub fragment: Option<Arc<ShaderModule>>,
    pub compute: Option<Arc<ShaderModule>>,
    compiler: Option<Compiler>,
    compiler_options: Option<CompileOptions<'a>>,
    device: Arc<Device>,
}

impl<'a> Shaders<'a> {
    pub fn new(device: Arc<Device>) -> Self {
        Self { vertex: None, fragment: None, compute: None, compiler: None, compiler_options: None, device }
    }

    pub fn load_shader_from_file(&mut self, path: impl AsRef<Path>, shader_type: &str) {
        let mut kind = ShaderKind::Compute;
        let mut shader_module: Option<Arc<ShaderModule>> = None;
        match shader_type {
            "vertex" => kind = ShaderKind::Vertex,
            "fragment" => kind = ShaderKind::Fragment,
            "compute" => kind = ShaderKind::Compute,
            _ => panic!("Invalid shader type"),
        }
        let spirv = self.compile_shader_from_file(path, kind);
        unsafe {
            shader_module = Some(ShaderModule::new(self.device.clone(), ShaderModuleCreateInfo::new(&spirv)).unwrap());
        }
        match shader_type {
            "vertex" => self.vertex = shader_module,
            "fragment" => self.fragment = shader_module,
            "compute" => self.compute = shader_module,
            _ => panic!("Invalid shader type"),
        }
    }

    pub fn load_shader_from_string(&mut self, source: &str, shader_type: &str) {
        let mut kind = ShaderKind::Compute;
        let mut shader_module: Option<Arc<ShaderModule>> = None;
        match shader_type {
            "vertex" => kind = ShaderKind::Vertex,
            "fragment" => kind = ShaderKind::Fragment,
            "compute" => kind = ShaderKind::Compute,
            _ => panic!("Invalid shader type"),
        }
        let spirv = self.compile_shader_from_string(source, kind);
        unsafe {
            shader_module = Some(ShaderModule::new(self.device.clone(), ShaderModuleCreateInfo::new(&spirv)).unwrap());
        }
        match shader_type {
            "vertex" => self.vertex = shader_module,
            "fragment" => self.fragment = shader_module,
            "compute" => self.compute = shader_module,
            _ => panic!("Invalid shader type"),
        }
    }

    fn compile_shader_from_file(&mut self, path: impl AsRef<Path>, kind: ShaderKind) -> Vec<u32> {
        if self.compiler.is_none() {
            self.compiler = Some(Compiler::new().unwrap());
        }
        let compiler = self.compiler.as_ref().unwrap();
        if self.compiler_options.is_none() {
            self.compiler_options = Some(CompileOptions::new().unwrap());
        }
        let mut options = self.compiler_options.as_mut().unwrap();
        let source = std::fs::read_to_string(path.as_ref()).unwrap();
        let compiled = compiler.compile_into_spirv(&source, kind, path.as_ref().to_str().unwrap(), "main", Some(&options)).unwrap();
        compiled.as_binary().to_vec()
    }

    fn compile_shader_from_string(&mut self, source: &str, kind: ShaderKind) -> Vec<u32> {
        if self.compiler.is_none() {
            self.compiler = Some(Compiler::new().unwrap());
        }
        let compiler = self.compiler.as_ref().unwrap();
        if self.compiler_options.is_none() {
            self.compiler_options = Some(CompileOptions::new().unwrap());
        }
        let mut options = self.compiler_options.as_mut().unwrap();
        let compiled = compiler.compile_into_spirv(&source, kind, "STRING_SOURCE", "main", Some(&options)).unwrap();
        compiled.as_binary().to_vec()
    }

    fn read_spirv_words_from_file(path: impl AsRef<Path>) -> Vec<u32> {
        // Taken from https://github.com/vulkano-rs/vulkano/blob/v0.34.0/examples/src/bin/runtime-shader/main.rs#L433
        let path = path.as_ref();
        let mut bytes = vec![];
        let mut file = File::open(path).unwrap_or_else(|err| {
            panic!(
                "can't open file `{}`: {}.\n\
                Note: this example needs to be run from the root of the example crate",
                path.display(),
                err,
            )
        });
        file.read_to_end(&mut bytes).unwrap();

        vulkano::shader::spirv::bytes_to_words(&bytes)
            .unwrap_or_else(|err| panic!("file `{}`: {}", path.display(), err))
            .into_owned()
    }
}