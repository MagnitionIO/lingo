use std::fs;
use std::io::Write;

use std::process::Command;

use crate::package::App;
use crate::util::execute_command_to_build_result;

use crate::backends::{
    BatchBackend, BatchBuildResults, BuildCommandOptions, BuildProfile, BuildResult, CommandSpec,
};

pub struct CmakeCpp;

fn gen_cmake_files(app: &App, options: &BuildCommandOptions) -> BuildResult {
    let build_dir = app.output_root.join("build");
    fs::create_dir_all(&build_dir)?;

    let src = &app.main_reactor;
    let src_folder = src.parent().expect("parent path is empty");
    let dst = app.output_root.join(src.file_name().expect("Main file name not found"));
    let dst_folder = dst.parent().expect("destination path is empty");

    // log::info!("src:{:?} dst:{:?} src_folder:{:?} dst_folder:{:?}", src, dst, src_folder, dst_folder);

    fs::copy(src, &dst)?;
    fs::copy(src_folder.join("CMakeLists.txt"), dst_folder.join("CMakeLists.txt"));

    // location of the cmake file
    let app_build_folder = dst_folder;
    let cmake_file = dst_folder.join("CMakeLists.txt");

    // log::info!("Cmake files app_build_folder:{:?} cmake_file:{:?}", app_build_folder, cmake_file);

    // create potential files that come from the target properties
    app.properties.write_artifacts(&app_build_folder)?;

    // we need to modify the cmake file here to include our generated cmake files
    let src_gen_dir = app.src_gen_dir();

    // read file and append cmake include to generated cmake file
    let mut content = fs::read_to_string(&cmake_file)?;

    let include_dir = "\ninclude_directories(lfc_include)";
    content += &*include_dir;

    let include_cmake = format!(
        "\ninclude({}/aggregated_cmake_include.cmake)",
        app_build_folder.display()
    );
    content += &*include_cmake;

    // overwrite cmake file
    let mut f = fs::OpenOptions::new().write(true).open(&cmake_file)?;
    f.write_all(content.as_ref())?;
    f.flush()?;

    // cmake args
    let mut cmake = Command::new("cmake");
    cmake.env("CMAKE_COLOR_MAKEFILE", "YES");
    cmake.arg(format!(
        "-DCMAKE_BUILD_TYPE={}",
        if options.profile == BuildProfile::Release {
            "RELEASE"
        } else {
            "DEBUG"
        }
    ));
    cmake.arg("-DCMAKE_INSTALL_BINDIR=bin");
    cmake.arg("-DREACTOR_CPP_VALIDATE=ON");
    cmake.arg("-DREACTOR_CPP_TRACE=OFF");
    cmake.arg("-DREACTOR_CPP_LOG_LEVEL=3");
    cmake.arg(dst_folder);
    cmake.arg(format!("-B {}", build_dir.display()));
    cmake.current_dir(&build_dir);

    // log::info!("cmake command:{:?}", cmake);

    execute_command_to_build_result(cmake)
}

fn do_cmake_build(results: &mut BatchBuildResults, options: &BuildCommandOptions) {
    // log::info!("CPP cmake build");
    // configure keep going parameter
    results.keep_going(options.keep_going);

    // // start code-generation
    // super::lfc::LFC::do_parallel_lfc_codegen(options, results, false);

    // log::info!("LFC is skipped, moving on options:{:?}", options);
    // stop process if the user request code-generation only
    if !options.compile_target_code {
        return;
    }

    results
        // generate all CMake files ahead of time
        .map(|app| gen_cmake_files(app, options))
        // Run cmake to build everything.
        .gather(|apps| {
            let build_dir = apps[0].output_root.join("build");

            // compile everything
            let mut cmake = Command::new("cmake");
            cmake.current_dir(&build_dir);
            cmake.env("CMAKE_COLOR_MAKEFILE", "YES");
            cmake.args(["--build", "."]);
            // for app in apps {
            //     // add one target arg for each app
            //     let name = app.main_reactor.file_stem().unwrap();
            //     cmake.arg("--target");
            //     cmake.arg(name);
            // }
            // note: by parsing CMake stderr we would know which specific targets have failed.
            execute_command_to_build_result(cmake)
        });
}

impl BatchBackend for CmakeCpp {
    fn execute_command(&mut self, command: &CommandSpec, results: &mut BatchBuildResults) {
        match command {
            CommandSpec::Build(options) => do_cmake_build(results, options),
            CommandSpec::Clean => {
                results.par_map(|app| {
                    crate::util::default_build_clean(&app.output_root)?;
                    Ok(())
                });
            }
            _ => todo!(),
        }
    }
}
