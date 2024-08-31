use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // 获取当前工作目录
    let cwd = env::current_dir().unwrap();
    // 定义源文件和目标文件的路径
    let readme_path = cwd.join("readme.md");
    let output_path = Path::new(&env::var("OUT_DIR").unwrap()).join("readme.md");

    // 复制文件到输出目录
    fs::copy(readme_path, output_path).expect("Failed to copy readme.md");
}