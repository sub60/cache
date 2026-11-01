use smol_str::SmolStr;

use crate::CommitSha;

/// A CI job, as identified by the claims its host signed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CiJob {
    /// The base URL of the host the job runs on, e.g. `https://github.com`.
    pub host: SmolStr,

    /// The software the host runs.
    pub host_software: HostSoftware,

    /// The host's ID for the namespace owning the job's repository, e.g.
    /// GitHub's `repository_owner_id`.
    pub host_namespace_id: SmolStr,

    /// The host's ID for the job's repository, e.g. GitHub's `repository_id`.
    pub host_repo_id: SmolStr,

    /// The name of the job's repository, without its namespace.
    pub host_repo_name: SmolStr,

    /// The Git ref the job runs on, e.g. `refs/heads/main`.
    pub git_ref: SmolStr,

    /// The commit the job runs on.
    pub commit_sha: CommitSha,

    /// The host's ID for the CI run the job belongs to, e.g. GitHub's
    /// `run_id`.
    pub host_run_id: SmolStr,

    /// Which attempt of the run the job belongs to, starting from 1, or `None`
    /// on hosts that don't number the attempts of a run.
    pub run_attempt: Option<u32>,

    /// The URL of the page of the run attempt.
    pub run_url: String,
}

/// The software a host runs.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum HostSoftware {
    /// GitHub.
    GitHub,
}
