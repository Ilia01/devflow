use anyhow::{Context, Result};
use colored::*;
use git2::Repository;

pub struct GitClient {
    repo: Repository,
}

impl GitClient {
    pub fn new() -> Result<Self> {
        let repo = Repository::open_from_env()
            .context("Not in a git repository. Make sure you're inside a git project.")?;

        Ok(Self { repo })
    }

    pub fn current_branch(&self) -> Result<String> {
        let head = self.repo.head().context("Failed to get HEAD reference")?;

        if head.is_branch() {
            let branch_name = head
                .shorthand()
                .context("Branch name contains invalid UTF-8")?
                .to_string();

            Ok(branch_name)
        } else {
            anyhow::bail!("Not currently on a branch (detached HEAD state)")
        }
    }

    pub fn create_branch(&self, branch_name: &str) -> Result<()> {
        let head_commit = self
            .repo
            .head()
            .context("Failed to get HEAD")?
            .peel_to_commit()
            .context("Failed to get HEAD commit")?;

        self.repo
            .branch(branch_name, &head_commit, false)
            .context(format!("Failed to create branch '{}'", branch_name))?;

        let refname = format!("refs/heads/{}", branch_name);

        self.repo
            .set_head(&refname)
            .context("Failed to set HEAD to new branch")?;

        self.repo
            .checkout_head(None)
            .context("Failed to checkout new branch")?;

        println!(
            "{}",
            format!("✓ Created and switched to branch '{}'", branch_name).green()
        );

        Ok(())
    }

    pub fn status_summary(&self) -> Result<String> {
        let statuses = self.repo.statuses(None)?;

        if statuses.is_empty() {
            return Ok("  Working directory clean".to_string());
        }

        let mut summary = Vec::new();

        for entry in statuses.iter() {
            let status = entry.status();
            let path = entry.path().unwrap_or("unknown");

            if status.is_wt_modified() {
                summary.push(format!("  {} {}", "M".yellow(), path));
            } else if status.is_wt_new() {
                summary.push(format!("  {} {}", "A".green(), path));
            } else if status.is_wt_deleted() {
                summary.push(format!("  {} {}", "D".red(), path));
            }
        }

        Ok(summary.join("\n"))
    }

    pub fn push(&self, branch_name: &str) -> Result<()> {
        let mut remote = self
            .repo
            .find_remote("origin")
            .context("Failed to find remote 'origin'")?;

        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });

        let mut push_options = git2::PushOptions::new();
        push_options.remote_callbacks(callbacks);

        remote
            .push(&[&refspec], Some(&mut push_options))
            .context(format!("Failed to push branch '{}'", branch_name))?;

        println!(
            "{}",
            format!("✓ Pushed branch '{}' to origin", branch_name).green()
        );

        Ok(())
    }

    pub fn commit(&self, message: &str) -> Result<()> {
        let mut index = self.repo.index().context("Failed to get repository index")?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
            .context("Failed to add files to index")?;
        index.write().context("Failed to write index")?;

        let tree_id = index.write_tree().context("Failed to write tree")?;
        let tree = self.repo.find_tree(tree_id).context("Failed to find tree")?;

        let head = self.repo.head().context("Failed to get HEAD")?;
        let parent_commit = head.peel_to_commit().context("Failed to get parent commit")?;

        let signature = self.repo.signature()
            .context("Failed to get git signature. Make sure git user.name and user.email are configured")?;

        self.repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                message,
                &tree,
                &[&parent_commit],
            )
            .context("Failed to create commit")?;

        println!("{}", format!("✓ Created commit: {}", message).green());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_client_in_repo() {
        let result = GitClient::new();
        assert!(result.is_ok(), "Should successfully open git repo");
    }

    #[test]
    fn test_current_branch() {
        if let Ok(git) = GitClient::new() {
            let result = git.current_branch();
            match result {
                Ok(branch) => assert!(!branch.is_empty(), "Branch name should not be empty"),
                Err(_) => {
                    // It's okay if we can't get branch (e.g., new repo with no commits)
                }
            }
        }
    }

    #[test]
    fn test_status_summary() {
        if let Ok(git) = GitClient::new() {
            let result = git.status_summary();
            assert!(result.is_ok(), "Should get status summary");
        }
    }
}
