use std::{
    io,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

#[derive(Debug, Clone)]
pub struct JJHandle {
    root: Arc<Path>,
}

impl Default for JJHandle {
    fn default() -> Self {
        Self {
            root: Path::new(".").into(),
        }
    }
}

impl JJHandle {
    pub fn current() -> io::Result<Self> {
        let output = Command::new("jj").arg("root").output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(io::Error::other(stderr.trim().to_owned()));
        }

        let root = String::from_utf8(output.stdout)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let root = PathBuf::from(root.trim_end());

        Ok(Self { root: root.into() })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn jj_joins(&self, cpts: impl IntoIterator<Item: AsRef<Path>>) -> PathBuf {
        let mut path = self.root().to_owned();
        path.push(".jj");
        for cpt in cpts {
            path.push(cpt);
        }
        path
    }
}
