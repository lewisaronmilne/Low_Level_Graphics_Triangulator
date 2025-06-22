fn main() 
{
    std::process::Command::new("glsl_to_spirv").arg("shaders/vert.glsl").arg("vert").arg("main")
        .output().expect("eRRoR: vertex shader failed to compile.");

    std::process::Command::new("glsl_to_spirv").arg("shaders/frag.glsl").arg("frag").arg("main")
        .output().expect("eRRoR: fragment shader failed to compile.");
}