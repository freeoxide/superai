//! Mutation-boundary failure injection (QAL-06): production paths take an
//! optional [`Injector`] fired at every [`Point`]; `None` costs one branch.

/// A boundary in the mutation pipeline that can fail. The set mirrors the
/// failure tests plus the §4.2 recheck; variant order is stable and each
/// point fires immediately before the step it names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Point {
    /// The target is opened/read for backup; any bytes are still untouched.
    BackupOpen,
    /// The backup copy is about to be written.
    BackupWrite,
    /// The landed backup is about to be flushed and synced.
    BackupFlush,
    /// The fresh backup is about to be re-read for its digest.
    BackupVerify,
    /// The exclusive same-directory temp is about to be created.
    TempCreate,
    /// Staged bytes are about to be written to the temp.
    TempWrite,
    /// The temp is about to be flushed and synced.
    TempFlush,
    /// Staged output is about to be parse-checked.
    ParseStaged,
    /// The prepare-to-commit recheck (§4.2): on-disk state vs the expectation.
    ConflictRecheck,
    /// The rename over the target is about to run.
    AtomicReplace,
    /// The parent directory is about to be synced.
    ParentSync,
    /// The committed file is about to be read back and digest-checked.
    ReadBackVerify,
    /// A rollback restore is about to be verified against its entry.
    RollbackVerify,
    /// Step index 1 of a multi-file transaction is about to commit.
    SecondFile,
    /// Step index 2 of a multi-file transaction is about to commit.
    ThirdFile,
    /// Journal written at `plan`; a failure simulates a crash there (MUT-09).
    JournalPlan,
    /// Journal advanced to `prepare_backup`; crash simulation.
    JournalPrepareBackup,
    /// Journal advanced to `stage_temp`; crash simulation.
    JournalStageTemp,
    /// Journal advanced to `commit`; crash simulation.
    JournalCommit,
    /// Journal advanced to `verify`; crash simulation.
    JournalVerify,
    /// Journal advanced to `rollback`; crash simulation.
    JournalRollback,
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::BackupOpen => "backup_open",
            Self::BackupWrite => "backup_write",
            Self::BackupFlush => "backup_flush",
            Self::BackupVerify => "backup_verify",
            Self::TempCreate => "temp_create",
            Self::TempWrite => "temp_write",
            Self::TempFlush => "temp_flush",
            Self::ParseStaged => "parse_staged",
            Self::ConflictRecheck => "conflict_recheck",
            Self::AtomicReplace => "atomic_replace",
            Self::ParentSync => "parent_sync",
            Self::ReadBackVerify => "read_back_verify",
            Self::RollbackVerify => "rollback_verify",
            Self::SecondFile => "second_file",
            Self::ThirdFile => "third_file",
            Self::JournalPlan => "journal_plan",
            Self::JournalPrepareBackup => "journal_prepare_backup",
            Self::JournalStageTemp => "journal_stage_temp",
            Self::JournalCommit => "journal_commit",
            Self::JournalVerify => "journal_verify",
            Self::JournalRollback => "journal_rollback",
        };
        f.write_str(s)
    }
}

/// Deterministic failure injection: `Err` simulates the boundary failing.
/// Implementations stay cheap and side-effect free apart from counters.
pub trait Injector: Send + Sync + std::fmt::Debug {
    /// Return `Err` to simulate `point` failing; `Ok` lets the pipeline
    /// continue.
    fn inject(&self, point: Point) -> crate::Result<()>;
}

/// Invoke an optional injector, short-circuiting on failure.
pub(crate) fn run(injector: Option<&dyn Injector>, point: Point) -> crate::Result<()> {
    if let Some(injector) = injector {
        injector.inject(point)
    } else {
        Ok(())
    }
}
