use std::env;
use std::process::Command;

// macro_rules! p {
//     ($($tokens: tt)*) => {
//         println!("cargo:warning={}", format!($($tokens)*))
//     }
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR").unwrap();

    // let npm = Command::new("npm")
    //     .args(["-v"])
    //     .output()
    //     .unwrap();

    // if !npm.stdout.is_empty() {
    //     let npm = String::from_utf8(npm.stdout).unwrap();
    //     if npm.is_empty() {
    //         p!("npm is required for installing proto files!");
    //     }
    // }

    Command::new("npm")
        .args(["i", "--prefix", &out_dir, "@a11ywatch/protos"])
        .output()
        .expect("failed to execute process");

    tonic_build::compile_protos(format!(
        "{}/node_modules/@a11ywatch/protos/crawler.proto",
        out_dir
    ))?;
    tonic_build::compile_protos(format!(
        "{}/node_modules/@a11ywatch/protos/website.proto",
        out_dir
    ))?;
    tonic_build::compile_protos(format!(
        "{}/node_modules/@a11ywatch/protos/health.proto",
        out_dir
    ))?;

    Ok(())
}
