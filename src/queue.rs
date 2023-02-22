// [[file:../qsubmit.note::137346a7][137346a7]]
use crate::common::*;
use std::os::unix::fs::PermissionsExt;
// 137346a7 ends here

// [[file:../qsubmit.note::*base][base:1]]

// base:1 ends here

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
