use std::{io, process::Stdio, string::FromUtf8Error, sync::LazyLock};

use ansi_to_tui::IntoText;
use bytestring::ByteString;
use compio::process::Command;
use ratatui::text::Text;
use regex::Regex;
use smol_str::SmolStr;

use crate::utils::jj::JJHandle;

impl JJHandle {
    pub async fn workspace_update_stale(&self) -> Result<(), CommandError> {
        let output = self
            .cmd_exec()
            .args(["workspace", "update-stale"])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(())
    }

    pub async fn log(&self, mode: &LogMode) -> Result<Vec<u8>, CommandError> {
        let mut cmd = self.cmd_read();

        cmd.args(["log", "--color=always"]);
        match mode {
            LogMode::Default => (),
            LogMode::Bookmark(v) => {
                cmd.args(["-r", &format!("..{v}")]);
            }
            LogMode::Tag(v) => {
                cmd.args(["-r", &format!("..{v}")]);
            }
        }

        let output = cmd.output().await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(output.stdout)
    }

    #[expect(clippy::new_ret_no_self)]
    pub async fn new(&self, parent: &str) -> Result<(), CommandError> {
        let output = self.cmd_exec().args(["new", parent]).output().await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(())
    }

    pub async fn edit(&self, id: &str) -> Result<(), CommandError> {
        let output = self.cmd_exec().args(["edit", id]).output().await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(())
    }

    pub async fn desc(&self, id: &str) -> Result<(), CommandError> {
        let output = self
            .cmd_wait()
            .args(["desc", "--editor", id])
            .output()
            .await?;

        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(())
    }

    pub async fn show(&self, revset: &str) -> Result<Vec<u8>, CommandError> {
        let output = self
            .cmd_read()
            .args(["show", "--color=always", "--stat", "-r", revset])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(output.stdout)
    }

    pub async fn diff_sum(&self, revset: &str) -> Result<Vec<u8>, CommandError> {
        let output = self
            .cmd_read()
            .args(["diff", "--color=always", "--summary", "-r", revset])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(output.stdout)
    }

    pub async fn diff(
        &self,
        revset: &str,
        status: &str,
        file: &str,
    ) -> Result<Vec<u8>, CommandError> {
        static RE_RENAME_DEST: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r#"(.*)\{.+ => (.+)\}(.*)"#).unwrap());

        let mut cmd = self.cmd_read();

        match status {
            "R" => {
                let file = RE_RENAME_DEST
                    .captures(file)
                    .map(|caps| format!("{}{}{}", &caps[1], &caps[2], &caps[3]))
                    .unwrap_or_else(|| file.into());

                cmd.args([
                    "diff",
                    "--color=always",
                    "--color-words",
                    "-r",
                    revset,
                    &file,
                ]);
            }
            _ => {
                cmd.args(["diff", "--color=always", "-r", revset, file]);
            }
        }

        let output = cmd.output().await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(output.stdout)
    }

    pub async fn abandon(&self, v: &Abandon) -> Result<(), CommandError> {
        let revset = match v {
            Abandon::One { id } => id.as_str(),
            Abandon::Range { start, end } => &format!("{start}::{end}"),
        };

        let output = self.cmd_exec().args(["abandon", revset]).output().await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(())
    }

    pub async fn split(&self, v: &Split) -> Result<(), CommandError> {
        let mut cmd = self.cmd_wait();
        cmd.args(["split", "--editor"]);
        if v.mode == SplitMode::Parallel {
            cmd.arg("--parallel");
        }
        let output = cmd.args(["-r", &v.id]).output().await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(())
    }

    pub async fn fetch(&self) -> Result<(), CommandError> {
        let output = self.cmd_exec().args(["git", "fetch"]).output().await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(())
    }

    pub async fn bookmarks(&self) -> Result<Vec<u8>, CommandError> {
        let output = self
            .cmd_read()
            .args([
                "bookmark",
                "list",
                "--color=always",
                "-T",
                r#"label("bookmark", name) ++ "\n""#,
            ])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(output.stdout)
    }

    pub async fn bookmarks_unsync(&self) -> Result<Vec<u8>, CommandError> {
        let output = self
            .cmd_read()
            .args([
                "bookmark",
                "list",
                "--color=always",
                "-T",
                r#"if(remote && !synced, concat(label("bookmark", name), " -> ", remote, "\n"))"#,
            ])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(output.stdout)
    }

    pub async fn bookmark_create(&self, id: &str, name: &str) -> Result<(), CommandError> {
        let output = self
            .cmd_exec()
            .args(["bookmark", "create", "-r", id, name])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(())
    }

    pub async fn bookmark_track(&self, name: &str, remote: &str) -> Result<(), CommandError> {
        let output = self
            .cmd_exec()
            .args(["bookmark", "track", name, "--remote", remote])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(())
    }

    pub async fn bookmark_untrack(&self, name: &str, remote: &str) -> Result<(), CommandError> {
        let output = self
            .cmd_exec()
            .args(["bookmark", "untrack", name, "--remote", remote])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(())
    }

    pub async fn bookmark_delete(&self, name: &str) -> Result<(), CommandError> {
        let output = self
            .cmd_exec()
            .args(["bookmark", "delete", name])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(())
    }

    pub async fn bookmark_set(&self, id: &str, name: &str) -> Result<(), CommandError> {
        let output = self
            .cmd_exec()
            .args(["bookmark", "set", "--allow-backwards", "-r", id, name])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(())
    }

    pub async fn bookmark_tree(&self) -> Result<String, CommandError> {
        let output = self
            .cmd_read()
            .args([
                "bookmark",
                "list",
                "--all-remotes",
                "-T",
                r#"if(remote && tracked, concat("  ", "@", remote, if(!synced, "*")), if(remote, concat(name, "@", remote), name)) ++ "\n""#,
            ])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).into())
    }

    pub async fn bookmark_remotes_untrack(
        &self,
        bookmark: &str,
    ) -> Result<Vec<SmolStr>, CommandError> {
        let remotes = self.remotes().await?;

        if remotes.is_empty() {
            return Ok(vec![]);
        }

        let output = self
            .cmd_read()
            .args([
                "bookmark",
                "list",
                "--all-remotes",
                "-T",
                &format!(r#"if(name == "{bookmark}" && remote, remote ++ "\n")"#),
            ])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }
        let remotes_track = String::from_utf8_lossy(&output.stdout);

        let remotes_untrack = String::from_utf8_lossy(&remotes)
            .lines()
            .filter_map(|line| {
                line.split_once(' ').and_then(|(remote, _)| {
                    remotes_track
                        .lines()
                        .all(|v| v != remote)
                        .then(|| remote.into())
                })
            })
            .collect();

        Ok(remotes_untrack)
    }

    pub async fn bookmark_remote_present(
        &self,
        name: &str,
        remote: &str,
    ) -> Result<bool, CommandError> {
        let output = self
            .cmd_read()
            .args([
                "bookmark",
                "list",
                "--all-remotes",
                "-T",
                &format!(r#"if(tracked && name == "{name}" && remote == "{remote}", present)"#),
            ])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout)
            .parse()
            .expect("must parse bool"))
    }

    pub async fn push_bookmark(&self, bookmark: &str, remote: &str) -> Result<(), CommandError> {
        let output = self
            .cmd_exec()
            .args(["git", "push", "--bookmark", bookmark, "--remote", remote])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(())
    }

    pub async fn push_tag(&self, name: &str, remote: &str) -> Result<(), CommandError> {
        let output = self
            .cmd_exec()
            .args(["git", "push", "--tag", name, "--remote", remote])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(())
    }

    pub async fn remotes(&self) -> Result<Vec<u8>, CommandError> {
        let output = self
            .cmd_read()
            .args(["git", "remote", "list"])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(output.stdout)
    }

    pub async fn tag_remotes_untrack(&self, tag: &str) -> Result<Vec<SmolStr>, CommandError> {
        let remotes = self.remotes().await?;

        if remotes.is_empty() {
            return Ok(vec![]);
        }

        let output = self
            .cmd_read()
            .args([
                "tag",
                "list",
                "--all-remotes",
                "-T",
                &format!(r#"if(name == "{tag}" && remote, remote ++ "\n")"#),
            ])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }
        let remotes_track = String::from_utf8_lossy(&output.stdout);

        let remotes_untrack = String::from_utf8_lossy(&remotes)
            .lines()
            .filter_map(|line| {
                line.split_once(' ').and_then(|(remote, _)| {
                    remotes_track
                        .lines()
                        .all(|v| v != remote)
                        .then(|| remote.into())
                })
            })
            .collect();

        Ok(remotes_untrack)
    }

    pub async fn tag_track(&self, name: &str, remote: &str) -> Result<(), CommandError> {
        let output = self
            .cmd_exec()
            .args(["tag", "track", name, "--remote", remote])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(())
    }

    pub async fn tag_untrack(&self, name: &str, remote: &str) -> Result<(), CommandError> {
        let output = self
            .cmd_exec()
            .args(["tag", "untrack", name, "--remote", remote])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(())
    }

    pub async fn tags(&self) -> Result<Vec<u8>, CommandError> {
        let output = self
            .cmd_read()
            .args([
                "tag",
                "list",
                "--color=always",
                "--all-remotes",
                "-T",
                r#"label("tag", name) ++ "\n""#,
            ])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(output.stdout)
    }

    pub async fn tag_set(&self, id: &str, name: &str) -> Result<(), CommandError> {
        let output = self
            .cmd_exec()
            .args(["tag", "set", "--allow-move", "-r", id, name])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(())
    }

    pub async fn tag_tree(&self) -> Result<String, CommandError> {
        let output = self
            .cmd_read()
            .args([
                "tag",
                "list",
                "--all-remotes",
                "-T",
                r#"if(remote && tracked, concat("  ", "@", remote, if(!synced, "*")), if(remote, concat(name, "@", remote), name)) ++ "\n""#,
            ])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).into())
    }

    pub async fn tag_delete(&self, tag: &str) -> Result<(), CommandError> {
        let output = self
            .cmd_exec()
            .args(["tag", "delete", tag])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(())
    }

    pub async fn tag_synced_remote(&self, name: &str, remote: &str) -> Result<bool, CommandError> {
        let output = self
            .cmd_read()
            .args([
                "tag",
                "list",
                "--all-remotes",
                "-T",
                &format!(r#"if(tracked && name == "{name}" && remote == "{remote}", synced)"#),
            ])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout)
            .parse()
            .expect("must parse bool"))
    }

    pub async fn tag_remote_present(&self, name: &str, remote: &str) -> Result<bool, CommandError> {
        let output = self
            .cmd_read()
            .args([
                "tag",
                "list",
                "--all-remotes",
                "-T",
                &format!(r#"if(tracked && name == "{name}" && remote == "{remote}", present)"#),
            ])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout)
            .parse()
            .expect("must parse bool"))
    }

    pub async fn undo(&self) -> Result<(), CommandError> {
        let output = self.cmd_exec().arg("undo").output().await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(())
    }

    pub async fn redo(&self) -> Result<(), CommandError> {
        let output = self.cmd_exec().arg("redo").output().await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(())
    }

    pub async fn expand_revset(&self, revset: &str) -> Result<Box<str>, ExpandRevsetError> {
        let output = self
            .cmd_read()
            .args([
                "log",
                "--no-graph",
                "--no-pager",
                "--color=never",
                "-r",
                revset,
                "-T",
                r#"if(divergent, concat(change_id, "/", change_offset), change_id) ++ "\n""#,
            ])
            .output()
            .await?;

        if !output.status.success() {
            return Err(ExpandRevsetError::Fail(output.stderr));
        } else if output.stdout.is_empty() {
            return Err(ExpandRevsetError::Invalid);
        }

        Ok(String::from_utf8(output.stdout)?.into())
    }

    pub async fn squash(&self, v: &Squash) -> Result<(), CommandError> {
        let mut cmd = self.cmd_wait();
        match v {
            Squash::ToParent { id } => cmd.args(["squash", "--editor", "-r", id]),
            Squash::ToStart { start, end } => cmd.args([
                "squash",
                "--editor",
                "-f",
                &format!("{start}..{end}"),
                "-t",
                start,
            ]),
            Squash::OneTo { from, to } => cmd.args(["squash", "--editor", "-f", from, "-t", to]),
            Squash::RangeTo { start, end, to } => cmd.args([
                "squash",
                "--editor",
                "-f",
                &format!("{start}::{end}"),
                "-t",
                to,
            ]),
        };
        let output = cmd.output().await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(())
    }

    pub async fn rebase(&self, v: &Rebase) -> Result<(), CommandError> {
        let mut cmd = self.cmd_exec();
        match v {
            Rebase::One { from, to } => cmd.args(["rebase", "-r", from, "--onto", to]),
            Rebase::Range { start, end, to } => {
                cmd.args(["rebase", "-r", &format!("{start}::{end}"), "--onto", to])
            }
        };

        let output = cmd.output().await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(())
    }

    pub async fn duplicate(&self, v: &Duplicate) -> Result<(), CommandError> {
        let mut cmd = self.cmd_exec();
        match v {
            Duplicate::One { from, to } => cmd.args(["duplicate", from, "--onto", to]),
            Duplicate::Range { start, end, to } => {
                cmd.args(["duplicate", &format!("{start}::{end}"), "--onto", to])
            }
        };

        let output = cmd.output().await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(())
    }

    pub async fn operations(&self) -> Result<Vec<u8>, CommandError> {
        let output = self
            .cmd_read()
            .args([
                "operation",
                "log",
                "--color=always",
                "--ignore-working-copy",
            ])
            .output()
            .await?;
        if !output.status.success() {
            return Err(CommandError::Fail(output.stderr));
        }

        Ok(output.stdout)
    }
}

impl JJHandle {
    fn cmd_read(&self) -> Command {
        let mut cmd = Command::new("jj");
        cmd.current_dir(self.root())
            .stdin(Stdio::null())
            .unwrap()
            .stdout(Stdio::piped())
            .unwrap()
            .stderr(Stdio::piped())
            .unwrap();
        cmd
    }

    fn cmd_exec(&self) -> Command {
        let mut cmd = Command::new("jj");
        cmd.current_dir(self.root())
            .stdin(Stdio::null())
            .unwrap()
            .stdout(Stdio::null())
            .unwrap()
            .stderr(Stdio::piped())
            .unwrap();
        cmd
    }

    fn cmd_wait(&self) -> Command {
        let mut cmd = Command::new("jj");
        cmd.current_dir(self.root()).stderr(Stdio::piped()).unwrap();
        cmd
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub enum LogMode {
    #[default]
    Default,
    Bookmark(ByteString),
    Tag(ByteString),
}

#[derive(Debug)]
pub struct Split {
    pub id: SmolStr,
    pub mode: SplitMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitMode {
    ParentChild,
    Parallel,
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("command executing error: {0}")]
    Io(#[from] io::Error),

    #[error("command executing failed: {}", String::from_utf8_lossy(&.0))]
    Fail(Vec<u8>),
}

#[derive(Debug, thiserror::Error)]
pub enum ExpandRevsetError {
    #[error("revset is invalid")]
    Invalid,

    #[error("command executing error: {0}")]
    Io(#[from] io::Error),

    #[error("command output/error is not UTF-8: {0}")]
    Utf8(#[from] FromUtf8Error),

    #[error("command executing failed: {}", String::from_utf8_lossy(&.0))]
    Fail(Vec<u8>),
}

#[derive(Debug)]
pub enum Abandon {
    One { id: SmolStr },
    Range { start: SmolStr, end: SmolStr },
}

impl Abandon {
    pub fn msg(&self) -> String {
        match self {
            Abandon::One { id } => format!("abandon `{id}`"),
            Abandon::Range { start, end } => format!("abandon `{start}::{end}`"),
        }
    }
}

#[derive(Debug)]
pub enum Squash {
    ToParent {
        id: SmolStr,
    },
    ToStart {
        start: SmolStr,
        end: SmolStr,
    },
    OneTo {
        from: SmolStr,
        to: SmolStr,
    },
    RangeTo {
        start: SmolStr,
        end: SmolStr,
        to: SmolStr,
    },
}

impl Squash {
    pub fn msg(&self) -> String {
        match self {
            Squash::ToParent { id } => {
                format!("squash `{id}` to parent")
            }
            Squash::ToStart { start, end } => format!("squash from `{start}..{end}` to `{start}`"),
            Squash::OneTo { from, to } => format!("squash from `{from}` to `{to}`"),
            Squash::RangeTo { start, end, to } => format!("squash from `{start}::{end}` to `{to}`"),
        }
    }
}

#[derive(Debug)]
pub enum Rebase {
    One {
        from: SmolStr,
        to: SmolStr,
    },
    Range {
        start: SmolStr,
        end: SmolStr,
        to: SmolStr,
    },
}

impl Rebase {
    pub fn msg(&self) -> String {
        match self {
            Rebase::One { from, to } => format!("rebase `{from}` to `{to}`"),
            Rebase::Range { start, end, to } => format!("rebase `{start}::{end}` to `{to}`"),
        }
    }

    pub fn reloc(&self) -> &str {
        match self {
            Rebase::One { from, .. } => from,
            Rebase::Range { end, .. } => end,
        }
    }
}

#[derive(Debug)]
pub enum Duplicate {
    One {
        from: SmolStr,
        to: SmolStr,
    },
    Range {
        start: SmolStr,
        end: SmolStr,
        to: SmolStr,
    },
}

impl Duplicate {
    pub fn msg(&self) -> String {
        match self {
            Duplicate::One { from, to } => format!("duplicate `{from}` to `{to}`"),
            Duplicate::Range { start, end, to } => format!("duplicate `{start}::{end}` to `{to}`"),
        }
    }
}

impl CommandError {
    pub fn into_text(self) -> Text<'static> {
        match self {
            CommandError::Fail(s) => s.into_text().unwrap_or_default(),
            e => e.to_string().into(),
        }
    }
}

impl ExpandRevsetError {
    pub fn into_text(self) -> Text<'static> {
        match self {
            ExpandRevsetError::Fail(s) => s.into_text().unwrap_or_default(),
            e => e.to_string().into(),
        }
    }
}
