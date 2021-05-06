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
    #[structopt(name = "QUEUE-DIR", long = "qdir", short = "q")]
    qdir: PathBuf,

    /// Continue to Watch queue dir for new jobs
    #[structopt(short)]
    watch: bool,
}

fn sleep(n: u64) {
    use std::{thread, time};

    let ten_millis = time::Duration::from_secs(n);
    let now = time::Instant::now();

    thread::sleep(ten_millis);
}

pub fn enter_main_loop() -> Result<()> {
    let args = Cli::from_args();
    args.verbose.setup_logger();

    let scan_rate = 2;
    loop {
        match get_next_job_from_qdir(&args.qdir, true) {
            Ok(q) => {
                let o = execute_job(dbg!(&q))?;
                dbg!(o);
            }
            Err(e) => {
                if args.watch {
                    info!("waiting {} seconds for new job ....", scan_rate);
                    sleep(scan_rate);
                } else {
                    return Err(e);
                }
            }
        }
    }
}
// cmd.rs:1 ends here
