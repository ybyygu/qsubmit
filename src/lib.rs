// [[file:../qsubmit.note::*imports][imports:1]]
use gut::prelude::*;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
// imports:1 ends here

// [[file:../qsubmit.note::*mods][mods:1]]
mod cmd;

pub use cmd::enter_main_loop;
// mods:1 ends here

// [[file:../qsubmit.note::*core][core:1]]
/// Return full path to an executable symlink from queue directory, ordered by
/// names
pub fn get_next_job_from_qdir(qdir: &Path) -> Result<PathBuf> {
    get_next_job_from_qdir_(qdir, true)
}

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

/// Exucute the script in a controlled way, returning script stdout
pub(crate) fn execute_job(job: &Path) -> Result<String> {
    assert!(!dbg!(job).is_dir());

    let wdir = job.parent().ok_or(format_err!("no parent dir?"))?;
    info!("Executing job: {:?}", job);
    info!("Working dir: {:?}", wdir);
    let out = duct::cmd!(job).dir(wdir).read()?;
    Ok(out)
}

/// Remove job from queue dir
pub(crate) fn archive_job(job: &Path) -> Result<()> {
    info!("archiving job {:?}", job);
    std::fs::remove_file(job).with_context(|| format!("failed to remove {:?}", job))?;
    Ok(())
}

fn is_executable(path: &Path) -> bool {
    if let Ok(metadata) = path.metadata() {
        metadata.permissions().mode() & 0o111 != 0
    } else {
        false
    }
}

#[test]
fn test_q() -> Result<()> {
    let q = get_next_job_from_qdir_("tests/queue".as_ref(), false)?;
    let q_expected: &Path = "tests/queue/00-real.sh".as_ref();
    assert_eq!(dbg!(&q), &q_expected.canonicalize()?);

    let out = execute_job(&q)?;
    let wdir: &Path = out.as_ref();
    let wdir_real: &Path = "tests/queue/adir".as_ref();
    assert_eq!(wdir, wdir_real.canonicalize()?);

    Ok(())
}
// core:1 ends here
