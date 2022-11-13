// [[file:../../qsubmit.note::d80b45c3][d80b45c3]]
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
    /// Display the log output of finished tasks on `node`
    Log {
        /// The node to connect
        node: String,
    },

    /// Display the current status of all tasks on `node`
    Status {
        /// The node to connect
        node: String,
    },

    /// Remotely shut down the daemon on `node`.
    Shutdown {
        /// The node to connect
        node: String,
    },

    /// List all jobs in queue-dir ~/gjf
    Jobs {},

    /// List available nodes for computations ~/.queues.
    Nodes {},

    /// Watch jobs in `~/gjf`
    Qdir {
        /// The node to connect
        node: String,
    },

    /// Enqueue the `script` to ~/gjf/
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
        SubTask::Log { node } => {
            let o = duct::cmd!("pueue", "-c", format!("{}/.queues/{}.yml", home, node,), "log").read()?;
            print!("{}", o);
        }
        SubTask::Status { node } => {
            let o = duct::cmd!("pueue", "-c", format!("{}/.queues/{}.yml", home, node,), "status").read()?;
            print!("{}", o);
        }
        SubTask::Shutdown { node } => {
            let o = duct::cmd!("pueue", "-c", format!("{}/.queues/{}.yml", home, node,), "shutdown").read()?;
            print!("{}", o);
        }
        SubTask::Qdir { node } => {
            let o = duct::cmd!(
                "pueue",
                "-c",
                format!("{}/.queues/{}.yml", home, node,),
                "add",
                format!("qsubmit -q {}/gjf -vv", home)
            )
            .read()?;
            print!("{}", o);
        }
        SubTask::Jobs {} => {
            let o = duct::cmd!("bash", "-c", "ls -l ~/gjf/").read()?;
            print!("{}", o);
        }
        SubTask::Jobs {} => {
            let o = duct::cmd!("bash", "-c", "ls -l ~/gjf/").read()?;
            print!("{}", o);
        }
        SubTask::Nodes {} => {
            let o = duct::cmd!("bash", "-c", "ls -l ~/.queues/Knode*.yml").read()?;
            print!("{}", o);
        }
        SubTask::Enqueue { script, qname } => {
            use std::os::unix::fs;

            let symlink = if let Some(qname) = qname {
                format!("{}/gjf/{}", home, qname)
            } else {
                format!("{}/gjf/{}", home, script)
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
// d80b45c3 ends here
