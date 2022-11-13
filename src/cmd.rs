// [[file:../qsubmit.note::69e4da5e][69e4da5e]]
use crate::*;
use gut::cli::*;
use gut::prelude::*;

/// A simple file queue system.
#[derive(StructOpt, Debug, Default)]
struct Cli {
    #[command(flatten)]
    verbose: gut::cli::Verbosity,

    /// The path to queue dir.
    #[arg(name = "QUEUE-DIR", long, short)]
    qdir: PathBuf,

    /// Continue to Watch queue dir for new jobs
    #[arg(short)]
    watch: bool,
}

pub fn enter_main_loop() -> Result<()> {
    let args = Cli::parse();
    args.verbose.setup_logger();

    let scan_rate = 2.0;
    loop {
        match get_next_job_from_qdir(&args.qdir) {
            Ok(q) => {
                let o = execute_job(dbg!(&q))?;
                dbg!(o);
            }
            Err(e) => {
                if args.watch {
                    info!("waiting {} seconds for new job ....", scan_rate);
                    gut::utils::sleep(scan_rate);
                } else {
                    return Err(e);
                }
            }
        }
    }
}
// 69e4da5e ends here
