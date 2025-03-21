use liblingo::args::TargetLanguage;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, io};

use clap::Parser;
use git2::BranchType::{Local, Remote};
use git2::{BranchType, Object, ObjectType, Reference, Repository};
use liblingo::args::InitArgs;
use liblingo::args::{BuildArgs, Command as ConsoleCommand, CommandLineArgs};
use liblingo::backends::{BatchBuildResults, BuildCommandOptions, CommandSpec};
use liblingo::package::tree::GitLock;
use liblingo::package::{Config, ConfigFile};
use liblingo::util::errors::{BuildResult, LingoError};
use liblingo::{GitCloneAndCheckoutCap, GitCloneError, GitUrl, WhichCapability, WhichError};
use log::LevelFilter;

fn do_which(cmd: &str) -> Result<PathBuf, WhichError> {
    which::which(cmd).map_err(|err| match err {
        which::Error::CannotFindBinaryPath => WhichError::CannotFindBinaryPath,
        which::Error::CannotGetCurrentDirAndPathListEmpty => {
            WhichError::CannotGetCurrentDirAndPathListEmpty
        }
        which::Error::CannotCanonicalize => WhichError::CannotCanonicalize,
    })
}

fn get_branch<'a>(
    repo: &'a git2::Repository,
    branch: &str,
    branch_type: BranchType,
) -> Result<(Object<'a>, Option<Reference<'a>>), GitCloneError> {
    let reference = repo
        .find_branch(branch, branch_type)
        .map_err(|_| GitCloneError("cannot find branch".to_string()))?
        .into_reference();
    Ok((
        reference
            .peel(ObjectType::Any)
            .map_err(|_| GitCloneError("cannot peel object".to_string()))?,
        Some(reference),
    ))
}

fn do_clone_and_checkout(
    git_url: GitUrl,
    outpath: &Path,
    git_tag: Option<GitLock>,
) -> Result<Option<String>, GitCloneError> {
    let repo = Repository::clone_recurse(<&str>::from(git_url), outpath)
        .map_err(|_| GitCloneError("clone failed".to_string()))?;
    let mut git_rev = None;

    if let Some(git_lock) = git_tag {
        let (object, reference) = match git_lock {
            GitLock::Tag(tag) => repo
                .revparse_ext(&tag)
                .map_err(|e| GitCloneError(format!("cannot parse rev {e}")))?,
            GitLock::Branch(branch) => {
                if let Ok(val) = get_branch(&repo, &branch, Local) {
                    val
                } else {
                    get_branch(&repo, &branch, Remote)?
                }
            }
            GitLock::Rev(rev) => repo
                .revparse_ext(&rev)
                .map_err(|e| GitCloneError(format!("cannot parse rev {e}")))?,
        };
        repo.checkout_tree(&object, None)
            .map_err(|e| GitCloneError(format!("cannot checkout rev {e}")))?;

        // TODO: this produces hard to debug output

        match reference {
            // gref is an actual reference like branches or tags
            Some(gref) => {
                git_rev = gref.target().map(|v| v.to_string());
                repo.set_head(gref.name().unwrap())
            }
            // this is a commit, not a reference
            None => repo.set_head_detached(object.id()),
        }
        .map_err(|_| GitCloneError("cannot checkout rev".to_string()))?;
    }

    Ok(git_rev)
}

fn do_read_to_string(p: &Path) -> io::Result<String> {
    std::fs::read_to_string(p)
}

fn main() {
    print_logger::new().level_filter(LevelFilter::Trace).init().unwrap();
    // parses command line arguments
    let args = CommandLineArgs::parse();
    // log::info!("arguments: {:?}", args);

    // Finds Lingo.toml recursively inside the parent directories.
    // If it exists the returned path is absolute.
    let lingo_path = liblingo::util::find_toml(&env::current_dir().unwrap());
    // log::info!("lingo toml file: {:?}", lingo_path);

    // tries to read Lingo.toml
    let mut wrapped_config: Option<Config> = lingo_path.as_ref().and_then(|path: &PathBuf| {
        ConfigFile::from(path, Box::new(do_read_to_string))
            .map_err(|err| log::error!("Error while reading Lingo.toml: {}", err))
            .ok()
            .map(|cf| cf.to_config(path.parent().unwrap()))
    });

    // log::info!("Toml config file:{:?}", wrapped_config);

    let result: BuildResult = validate(&mut wrapped_config, &args.command);
    if result.is_err() {
        print_res(result)
    }

    // log::info!("After Validation -- Toml config file:{:?}", wrapped_config);

    let result = execute_command(
        &mut wrapped_config,
        args.command,
        Box::new(do_which),
        Box::new(do_clone_and_checkout),
    );

    match result {
        CommandResult::Batch(res) => res.print_results(),
        CommandResult::Single(res) => print_res(res),
    }
}

fn print_res(result: BuildResult) {
    match result {
        Ok(_) => {
            log::info!("Single Processing: Success!");
        }
        Err(errs) => {
            log::error!("{}", errs);
        }
    }
}

fn validate(config: &mut Option<Config>, command: &ConsoleCommand) -> BuildResult {
    match (config, command) {
        (Some(config), ConsoleCommand::Build(build))
        | (Some(config), ConsoleCommand::Run(build)) => {
            let unknown_names = build
                .apps
                .iter()
                .filter(|&name| !config.apps.iter().any(|app| &app.name == name))
                .cloned()
                .collect::<Vec<_>>();
            if !unknown_names.is_empty() {
                return Err(Box::new(LingoError::UnknownAppNames(unknown_names)));
            }
            // Now remove the apps that were not selected by the CLI
            if !build.apps.is_empty() {
                config.apps.retain(|app| build.apps.contains(&app.name));
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn execute_command<'a>(
    config: &'a mut Option<Config>,
    command: ConsoleCommand,
    _which_capability: WhichCapability,
    git_clone_capability: GitCloneAndCheckoutCap,
) -> CommandResult<'a> {
    match (config, command) {
        (_, ConsoleCommand::Init(init_config)) => {
            // log::info!("Executing Init Command");
            CommandResult::Single(do_init(init_config, &git_clone_capability))
        }
        (None, _) => CommandResult::Single(Err(Box::new(io::Error::new(
            ErrorKind::NotFound,
            "Error: Missing Lingo.toml file",
        )))),
        (Some(config), ConsoleCommand::Build(build_command_args)) => {
            CommandResult::Batch(build(&build_command_args, config))
        }
        (Some(config), ConsoleCommand::Run(build_command_args)) => {
            let mut res = build(&build_command_args, config);
            res.map(|app| {
                // let mut command = Command::new(app.executable_path());
                let mut command = Command::new(app.output_root.join("build").join(app.name.clone()));
                let cfg_file = build_command_args.config_file.as_ref().map(|path| path.to_str().unwrap_or_default());
                let visualize = build_command_args.visualize;

                command.arg(visualize.to_string());

                if let Some(cfg_file_str) = cfg_file {
                    command.arg(cfg_file_str);  // Add the config file path if it's present
                } else {
                    command.arg("");
                }

                // log::info!("running command:{:?}", command);
                liblingo::util::run_and_capture(&mut command)?;
                Ok(())
            });
            CommandResult::Batch(res)
        }
        (Some(config), ConsoleCommand::Clean) => {
            CommandResult::Batch(run_command(CommandSpec::Clean, config, true))
        }
        _ => todo!(),
    }
}

fn do_init(init_config: InitArgs, git_clone_capability: &GitCloneAndCheckoutCap) -> BuildResult {
    // log::info!("Doing Init");
    if Path::new("Lingo.toml").exists() {
        log::info!("Already initialized, no need to do anything else.");
        return Ok(()); // Return early if Lingo.toml exists
    }

    let initial_config = ConfigFile::new_for_init_task(&init_config)?;
    // log::info!("Creating Files!");
    initial_config.write(Path::new("./Lingo.toml"))?;
    // log::info!("Written Toml File!");
    initial_config.setup_example(
        init_config.platform,
        init_config.language.unwrap_or(TargetLanguage::Cpp),
        git_clone_capability,
    )
}

fn build<'a>(args: &BuildArgs, config: &'a mut Config) -> BatchBuildResults<'a> {
    run_command(
        CommandSpec::Build(BuildCommandOptions {
            profile: args.build_profile(),
            compile_target_code: !args.no_compile,
            lfc_exec_path: "".into(),
            max_threads: args.threads,
            keep_going: args.keep_going,
        }),
        config,
        args.keep_going,
    )
}

fn run_command(task: CommandSpec, config: &mut Config, _fail_at_end: bool) -> BatchBuildResults {
    let _apps = config.apps.iter().collect::<Vec<_>>();
    liblingo::backends::execute_command(
        &task,
        config,
        Box::new(do_which),
        Box::new(do_clone_and_checkout),
    )
}

enum CommandResult<'a> {
    Batch(BatchBuildResults<'a>),
    Single(BuildResult),
}
