// [[file:../qsubmit.note::137346a7][137346a7]]
use crate::common::*;
use std::os::unix::fs::PermissionsExt;
// 137346a7 ends here

// [[file:../qsubmit.note::13360946][13360946]]
fn get_next_job_from_qdir_(qdir: &Path, archive: bool) -> Result<PathBuf> {
    // ignore sub directories
    let mut queued = vec![];

    let entries = std::fs::read_dir(qdir).with_context(|| format!("Invalid queue dir: {:?}", qdir))?;
    for entry in entries {
        // Here, `entry` is a `DirEntry`.
        let entry = entry?;
        let file_type = entry
            .file_type()
            .with_context(|| format!("Couldn't get file type for {:?}", entry.path()))?;

        let path = entry.path();
        // get execuable scripts only
        if file_type.is_symlink() && is_executable(&path) {
            queued.push(path);
        }
    }
    info!("Found {} queued jobs in {:?}", queued.len(), qdir);
    queued.sort();
    queued.reverse();

    // NOTE: If the job removed by others instantly, we simply ignore it and try
    // the next one
    while let Some(f) = queued.pop() {
        // NOTE: symbolic links will be resolved
        match f.canonicalize() {
            Ok(f_real) => {
                // remove this job in queue to avoid double-submission
                if archive {
                    archive_job(&f)?;
                }
                return Ok(f_real);
            }
            Err(e) => {
                error!("get queued job error: {:?}", e);
            }
        }
    }
    bail!("No queued job found in {:?}", qdir)
}

fn is_executable(path: &Path) -> bool {
    if let Ok(metadata) = path.metadata() {
        metadata.permissions().mode() & 0o111 != 0
    } else {
        false
    }
}

/// Remove job from queue dir
fn archive_job(job: &Path) -> Result<()> {
    info!("archiving job {:?}", job);
    std::fs::remove_file(job).with_context(|| format!("failed to remove {:?}", job))?;
    Ok(())
}

#[test]
fn test_q() -> Result<()> {
    let q = get_next_job_from_qdir_("tests/queue".as_ref(), false)?;
    let q_expected: &Path = "tests/queue/00-real.sh".as_ref();
    assert_eq!(&q, &q_expected.canonicalize()?);

    let out = execute_job(&q)?;
    let wdir: &Path = out.as_ref();
    let wdir_real: &Path = "tests/queue/adir".as_ref();
    assert_eq!(wdir, wdir_real.canonicalize()?);

    Ok(())
}
// 13360946 ends here

// [[file:../qsubmit.note::043e97ca][043e97ca]]
/// Exucute the script in a controlled way, returning script stdout
pub(crate) fn execute_job(job: &Path) -> Result<String> {
    assert!(!job.is_dir());

    let wdir = job.parent().ok_or(format_err!("no parent dir?"))?;
    info!("Executing job: {:?}", job);
    info!("Working dir: {:?}", wdir);
    let out = duct::cmd!(job).dir(wdir).read()?;
    Ok(out)
}

/// Return full path to an executable symlink from queue directory, ordered by
/// names
pub fn get_next_job_from_qdir(qdir: &Path) -> Result<PathBuf> {
    get_next_job_from_qdir_(qdir, true)
}
// 043e97ca ends here

// [[file:../qsubmit.note::24a8bc2e][24a8bc2e]]
/// A simple job queue system for submission of files in a directory.
#[derive(Debug)]
pub struct JobFileQueue {
    /// The path to queue dir.
    qdir: PathBuf,

    /// Continue to Watch queue dir for new jobs
    watch: bool,

    /// Wait `scan_delay` seconds periodically for new job if `qdir`
    /// is empty
    scan_delay: f64,
}

impl JobFileQueue {
    /// Construct `JobFileQueue` from queue directory in `qdir`.
    pub fn from_path(qdir: &Path) -> Self {
        Self {
            qdir: qdir.to_owned(),
            watch: true,
            scan_delay: 2.0,
        }
    }

    /// Set scan delay for new job in queue directory.
    pub fn set_scan_delay(&mut self, delay: f64) {
        assert!(delay.is_sign_positive(), "invalid delay: {delay}");
        self.scan_delay = delay;
    }
}

impl JobFileQueue {
    /// Return full path to an executable symlink from queue
    /// directory, ordered by names
    pub fn get_next_job(&self) -> Result<PathBuf> {
        let scan_delay = self.scan_delay;
        loop {
            match get_next_job_from_qdir(&self.qdir) {
                Ok(q) => break Ok(q),
                Err(e) => {
                    if self.watch {
                        info!("waiting {scan_delay} seconds for new job ....");
                        gut::utils::sleep(scan_delay);
                    } else {
                        break Err(e);
                    }
                }
            }
        }
    }

    /// Enqueue the `script` to directory as a symlink in `name`.
    pub fn enqueue<'a>(&self, script: &Path, name: impl Into<Option<&'a Path>>) -> Result<()> {
        use std::os::unix::fs;
        ensure!(script.is_file(), "invalid script path: {:?}", script);

        let mut symlink = self.qdir.to_owned();
        if let Some(name) = name.into() {
            symlink.push(name);
        } else {
            let file = script.file_name().unwrap();
            symlink.push(file);
        };
        let script = Path::new(&script).canonicalize()?;
        info!("enqueue {script:?} as {symlink:?}");
        fs::symlink(script, symlink)?;
        Ok(())
    }
}
// 24a8bc2e ends here
