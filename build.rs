use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;

use shaderc;

#[derive(Debug)]
struct ModuleDescriptor {
    input_file_name: String,
    output_file_name: String,
}

fn extract_shader_kind(path_str: &str) -> Option<shaderc::ShaderKind> {
    if path_str.ends_with(".vs.glsl") {
        return Some(shaderc::ShaderKind::Vertex);
    } else if path_str.ends_with(".fs.glsl") {
        return Some(shaderc::ShaderKind::Fragment);
    } else if path_str.ends_with(".cs.glsl") {
        return Some(shaderc::ShaderKind::Compute);
    }

    return None;
}

fn build_shaders(
    folder: &OsString,
    include_root: &OsString,
    out_dir: &OsString,
    compiler: &shaderc::Compiler,
    modules: &mut Vec<ModuleDescriptor>,
) {
    println!("scanning '{}'", folder.to_string_lossy());

    fs::create_dir_all(Path::new(&out_dir).join(folder)).unwrap();

    for file in fs::read_dir(folder).unwrap() {
        let source_path = file.unwrap().path();
        if source_path.is_dir() {
            let subfolder = OsString::from(source_path);
            build_shaders(&subfolder, include_root, out_dir, compiler, modules);
            continue;
        }

        let source_path_str = source_path.to_string_lossy();
        let source_text = fs::read_to_string(&source_path).unwrap();

        let kind = extract_shader_kind(&source_path_str);
        if kind == None {
            continue;
        }

        let mut compile_options = shaderc::CompileOptions::new().unwrap();
        compile_options.set_include_callback(
            |filename: &str,
             _include_type: shaderc::IncludeType,
             parent: &str,
             _include_depth: usize|
             -> shaderc::IncludeCallbackResult {
                let content = fs::read_to_string(Path::new(include_root).join(filename));

                if content.is_err() {
                    panic!(
                        "Failed to load include \"{}\" from \"{}\"",
                        filename, parent
                    );
                }

                Ok(shaderc::ResolvedInclude {
                    resolved_name: filename.into(),
                    content: content.unwrap(),
                })
            },
        );

        let spirv = compiler
            .compile_into_spirv(
                &source_text,
                kind.unwrap(),
                &source_path_str,
                "main",
                Some(&compile_options),
            )
            .unwrap();

        let output_path = Path::new(&out_dir).join(source_path.with_extension("spv"));

        println!("writing {}", output_path.to_string_lossy());
        fs::write(&output_path, spirv.as_binary_u8()).unwrap();

        let input_file_name = source_path
            .strip_prefix(include_root)
            .unwrap()
            .to_string_lossy()
            .to_string()
            .replace("\\", "/");

        let output_file_name = output_path
            .strip_prefix(out_dir)
            .unwrap()
            .to_string_lossy()
            .to_string();

        modules.push(ModuleDescriptor {
            input_file_name,
            output_file_name,
        });
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=shaders");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let compiler = shaderc::Compiler::new().unwrap();
    let mut modules = Vec::new();

    let root_folder = OsString::from("shaders");
    build_shaders(
        &root_folder,
        &root_folder,
        &out_dir,
        &compiler,
        &mut modules,
    );

    println!("Modules: {:?}", modules);
    let modules_file_path = Path::new(&out_dir).join("shader_modules.rs");
    let mut modules_file = BufWriter::new(fs::File::create(modules_file_path).unwrap());

    writeln!(
        &mut modules_file,
        "
        use std::collections::HashMap;

        pub type ShaderModules = HashMap<&'static str, wgpu::ShaderModule>;

        pub fn load(device: &wgpu::Device) -> ShaderModules {{
            HashMap::from([
        "
    )
    .unwrap();

    for module in modules {
        writeln!(
            &mut modules_file,
            "(\"{}\", device.create_shader_module(wgpu::ShaderModuleDescriptor {{
                label: Some(\"{}\"),
                source: wgpu::util::make_spirv(include_bytes!(r\"{}\")),
            }})),",
            module.input_file_name, module.input_file_name, module.output_file_name
        )
        .unwrap();
    }
    writeln!(
        &mut modules_file,
        "   ])
        }}"
    )
    .unwrap();
}
