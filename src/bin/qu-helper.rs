// [[file:../../qsubmit.note::1502f0e9][1502f0e9]]
use gut::cli::*;
use gut::prelude::*;
use std::path::{Path, PathBuf};

/// A simple file queue system.
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    task: SubTask,
}

#[derive(Parser)]
enum SubTask {
    /// List all jobs in queue-dir ~/jobs
    #[command(alias="ls")]
    ListJobs {},

    /// Enqueue the `script` to ~/jobs
    Enqueue {
        /// The script file to enqueue
        script: String,
        /// The name of symbolic link
        qname: Option<String>,
    },

    /// Generate bash shell completion script
    Init {},
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let home = std::env::var("HOME").unwrap();
    match args.task {
        SubTask::ListJobs {} => {
            let o = duct::cmd!("bash", "-c", "ls -l ~/jobs/").read()?;
            println!("{}", o);
        }
        SubTask::Enqueue { script, qname } => {
            use std::os::unix::fs;

            let symlink = if let Some(qname) = qname {
                format!("{}/jobs/{}", home, qname)
            } else {
                format!("{}/jobs/{}", home, script)
            };
            fs::symlink(Path::new(&script).canonicalize()?, dbg!(symlink))?;
        }
        // Generate bash completion script
        SubTask::Init {} => {
            use clap_complete::{generate, shells::Bash};

            let mut app = Cli::command();
            generate(Bash, &mut app, env!("CARGO_BIN_NAME"), &mut std::io::stdout());
        }
        _ => {
            todo!();
        }
    }

    Ok(())
}
// 1502f0e9 ends here
