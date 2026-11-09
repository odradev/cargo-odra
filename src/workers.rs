//! Running a step for every contract, several at a time.
//!
//! Every contract is a separate `cargo build` of the same crate, since the contract is chosen
//! with a cfg flag, and Cargo locks a target directory for the duration of a build. So the
//! workers each own a target directory: worker 0 the project's, like a single job always did,
//! the others one of their own under `target/cargo-odra`.

use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::Mutex,
    thread,
};

use crate::{errors::Error, log, paths};

/// Most contracts processed at the same time when `--jobs` is not given.
const MAX_DEFAULT_JOBS: usize = 8;

/// How many contracts to process at the same time on a machine with `cpus` CPUs when the user
/// does not say: half of them, at least one and at most [`MAX_DEFAULT_JOBS`].
///
/// A contract build keeps less than one CPU busy on average (the crate compiles with a single
/// codegen unit and links with fat LTO, with cargo's own work in between), so on 4 CPUs four
/// jobs still finish 17 contracts in a third of the time of one. Every job past the first owns
/// a target directory with its own copy of the dependencies though, a few hundred MB that are
/// compiled once more on first use, so the default takes half the CPUs rather than all.
fn default_jobs(cpus: usize) -> usize {
    (cpus / 2).clamp(1, MAX_DEFAULT_JOBS)
}

/// The workers of a project: how many, and where they build.
pub struct Workers {
    jobs: usize,
    target_dir: PathBuf,
}

/// Where one worker runs cargo.
pub struct Worker {
    /// The target directory cargo builds into.
    pub target_dir: PathBuf,
    /// Whether the target directory is the project's own, which cargo uses without being told.
    is_project_target_dir: bool,
    /// Whether other workers run at the same time.
    parallel: bool,
}

impl Worker {
    /// What to pass cargo as `--target-dir`: nothing for the project's own target directory.
    pub fn cargo_target_dir(&self) -> Option<&Path> {
        (!self.is_project_target_dir).then_some(self.target_dir.as_path())
    }

    /// Whether cargo's output should be held back and written out in one piece, so the output
    /// of the workers does not interleave.
    pub fn capture_output(&self) -> bool {
        self.parallel
    }
}

impl Workers {
    /// `jobs` workers, or the default for this machine ([`default_jobs`]) when not given.
    pub fn new(jobs: Option<usize>, project_root: &Path) -> Self {
        let jobs = jobs.unwrap_or_else(|| {
            default_jobs(thread::available_parallelism().map_or(1, |cpus| cpus.get()))
        });
        Workers {
            jobs: jobs.max(1),
            target_dir: paths::build_target_dir(project_root),
        }
    }

    /// Runs `step` for every item, as many at a time as there are workers, and never more than
    /// there are items.
    ///
    /// The items wait in a queue the workers take from, so a slow one does not hold up the
    /// others. The first failure stops the workers once each has finished the step it is on,
    /// and is returned after they have. `action` names what is done, for the log: "Building".
    pub fn run<T, F>(&self, action: &str, items: Vec<T>, step: F) -> Result<(), Error>
    where
        T: Send,
        F: Fn(&T, &Worker) -> Result<(), Error> + Sync,
    {
        let jobs = self.jobs.min(items.len()).max(1);
        if jobs > 1 {
            log::info(format!(
                "{action} {} contracts, {jobs} at a time...",
                items.len()
            ));
        }

        let queue = Mutex::new(VecDeque::from(items));
        let failure: Mutex<Option<Error>> = Mutex::new(None);
        thread::scope(|scope| {
            for index in 0..jobs {
                let worker = self.worker(index, jobs > 1);
                let (queue, failure, step) = (&queue, &failure, &step);
                scope.spawn(move || {
                    if !worker.is_project_target_dir && !worker.target_dir.exists() {
                        log::info(format!(
                            "Build directory {} is new; its first build compiles the dependencies.",
                            worker.target_dir.display()
                        ));
                    }
                    loop {
                        if failure.lock().unwrap().is_some() {
                            return;
                        }
                        let Some(item) = queue.lock().unwrap().pop_front() else {
                            return;
                        };
                        if let Err(error) = step(&item, &worker) {
                            *failure.lock().unwrap() = Some(error);
                            return;
                        }
                    }
                });
            }
        });
        match failure.into_inner().unwrap() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    fn worker(&self, index: usize, parallel: bool) -> Worker {
        match paths::worker_target_dir(&self.target_dir, index) {
            Some(target_dir) => Worker {
                target_dir,
                is_project_target_dir: false,
                parallel,
            },
            None => Worker {
                target_dir: self.target_dir.clone(),
                is_project_target_dir: true,
                parallel,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        path::{Path, PathBuf},
        sync::Mutex,
    };

    use super::{default_jobs, Workers, MAX_DEFAULT_JOBS};
    use crate::errors::Error;

    #[test]
    fn default_jobs_is_half_the_cpus_within_bounds() {
        assert_eq!(default_jobs(1), 1);
        assert_eq!(default_jobs(2), 1);
        assert_eq!(default_jobs(3), 1);
        // A 4-CPU CI runner builds two at a time.
        assert_eq!(default_jobs(4), 2);
        assert_eq!(default_jobs(8), 4);
        assert_eq!(default_jobs(16), MAX_DEFAULT_JOBS);
        assert_eq!(default_jobs(128), MAX_DEFAULT_JOBS);
    }

    fn workers(jobs: usize) -> Workers {
        Workers {
            jobs,
            target_dir: PathBuf::from("/project/target"),
        }
    }

    #[test]
    fn every_item_is_processed_once_and_worker_zero_uses_the_project_target_dir() {
        let seen = Mutex::new(Vec::new());
        workers(3)
            .run("Testing", (0..10).collect(), |item, worker| {
                let parallel_dir = worker.cargo_target_dir().map(Path::to_path_buf);
                assert!(worker.capture_output());
                assert_eq!(
                    parallel_dir.is_none(),
                    worker.target_dir == Path::new("/project/target")
                );
                seen.lock().unwrap().push(*item);
                Ok(())
            })
            .unwrap();
        let mut seen = seen.into_inner().unwrap();
        seen.sort();
        assert_eq!(seen, (0..10).collect::<Vec<_>>());
    }

    #[test]
    fn a_single_job_builds_in_the_project_target_dir_without_capturing_output() {
        workers(1)
            .run("Testing", vec![()], |_, worker| {
                assert_eq!(worker.cargo_target_dir(), None);
                assert!(!worker.capture_output());
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn the_first_failure_is_returned_and_stops_the_queue() {
        let processed = Mutex::new(0);
        let result = workers(2).run("Testing", (0..100).collect(), |item, _| {
            *processed.lock().unwrap() += 1;
            if *item == 3 {
                Err(Error::CommandFailed("item 3".to_string()))
            } else {
                Ok(())
            }
        });
        assert!(matches!(result, Err(Error::CommandFailed(message)) if message == "item 3"));
        // Far from all 100 items were taken from the queue after the failure.
        assert!(*processed.lock().unwrap() < 50);
    }
}
