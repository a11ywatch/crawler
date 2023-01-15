use std::env;
use std::process::Command;

#[allow(missing_docs)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR").unwrap();

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

    // os info validate protbuf install. If not installed - install and remove after.
    let info = os_info::get();
    let os_type = info.os_type();

    // check if protoc is installed
    let protoc_installed = match Command::new("protoc").arg("--version").output() {
        Ok(_) => true,
        _ => false,
    };

    // TODO: use binary 
    // curl -LO https://github.com/protocolbuffers/protobuf/releases/download/v3.15.8/protoc-3.15.8-linux-x86_64.zip
    // unzip protoc-3.15.8-linux-x86_64.zip -d $HOME/.local
    // export PATH="$PATH:$HOME/.local/bin"
    // OR git clone source
    // git clone https://github.com/protocolbuffers/protobuf
    // cd protobuf
    // git submodule update --init --recursive
    // cmake --build .

    if !protoc_installed {
        if os_type == os_info::Type::Ubuntu
            || os_type == os_info::Type::Pop
            || os_type == os_info::Type::Debian
        {
            match Command::new("apt")
                .args(["install", "-y", "protobuf-compiler", "libprotobuf-dev"])
                .output()
            {
                Ok(_) => {}
                _ => {
                    println!("failed to install protobuf-compiler and libprotobuf-dev");
                }
            };
        }
        if os_type == os_info::Type::FreeBSD {
            match Command::new("pkg").args(["install", "protobuf"]).output() {
                Ok(_) => {}
                _ => {
                    println!("failed to install protobuf");
                }
            };
        }
        if os_type == os_info::Type::Alpine {
            match Command::new("apk")
                .args(["add", "protoc", "protobuf-dev"])
                .output()
            {
                Ok(_) => {}
                _ => {
                    println!("failed to install protoc and protobuf-dev");
                }
            };
        }
        if os_type == os_info::Type::CentOS {
            match Command::new("yum").args(["install", "protobuf"]).output() {
                Ok(_) => {}
                _ => {
                    println!("failed to install protobuf");
                }
            };
        }
        if os_type == os_info::Type::Fedora {
            match Command::new("snap")
                .args(["install", "protobuf", "--classic"])
                .output()
            {
                Ok(_) => {}
                _ => {
                    println!("failed to install protobuf from snap");
                }
            };
        }

        // brew install protobuf ( does not accept xcode licenses )
        if os_type == os_info::Type::Macos {
            match Command::new("brew").arg("--version").output() {
                Ok(_) => {}
                _ => {
                    match Command::new("curl")
                        .args([
                            "-fsSL",
                            "-o",
                            "install.sh",
                            "https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh",
                        ])
                        .output()
                    {
                        Ok(_) => {
                            match Command::new("./install.sh").output() {
                                Ok(_) => {
                                    match Command::new("echo")
                                        .args([
                                            "export PATH=/usr/local/bin:$PATH",
                                            ">",
                                            "~/.bash_profile",
                                        ])
                                        .output()
                                    {
                                        Ok(_) => {}
                                        _ => {
                                            println!("failed to update $PATH");
                                        }
                                    };
                                }
                                _ => {
                                    println!("failed to run install.sh");
                                }
                            };
                        }
                        _ => {
                            println!("failed to curl brew install.sh");
                        }
                    };
                }
            };

            match Command::new("brew").args(["install", "protobuf"]).output() {
                Ok(_) => {}
                _ => {
                    println!("failed to install protobuf");
                }
            };
        }
        if os_type == os_info::Type::Windows {
            // todo: check 32 bit or 64 bit
            // protoc-21.12-win32.zip
            // protoc-21.12-win64.zip
            // download and extract to path
            println!("windows installation wip.");
        }
    }

    Ok(())
}
