// [[file:../qsubmit.note::*cmd.rs][cmd.rs:1]]
use crate::*;
use gut::cli::*;
use gut::prelude::*;
use structopt::*;

/// A simple file queue system.
#[derive(StructOpt, Debug, Default)]
struct Cli {
    #[structopt(flatten)]
    verbose: gut::cli::Verbosity,

    /// The path to queue dir.
    #[structopt(name = "QUEUE-DIR", long)]
    qdir: PathBuf,
}

pub fn enter_main_loop() -> Result<()> {
    let args = Cli::from_args();
    args.verbose.setup_logger();

    let qdir = &args.qdir;
    loop {
        let q = get_next_job_from_qdir(qdir, true)?;
        let o = execute_job(&q)?;
        archive_job(&q)?;
        dbg!(o);
    }
}
// cmd.rs:1 ends here
