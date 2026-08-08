//! A tiny in-memory virtual filesystem shared by every app that wants one
//! (Finder browses it, Terminal manipulates it, Preview reads from it).
//! Paths are absolute, `/`-separated, normalized (no `.`, no trailing
//! slash except root). Nodes are stored flat, keyed by full path — a
//! directory's children are just every key sharing its `path/` prefix
//! with no further `/` in the remainder, which `BTreeMap::range` makes
//! cheap to enumerate in sorted order without a real tree.

use std::collections::BTreeMap;

pub enum VfsNode {
    Dir,
    File(String),
}

pub struct Vfs {
    nodes: BTreeMap<String, VfsNode>,
}

impl Default for Vfs {
    fn default() -> Self {
        Self::new()
    }
}

impl Vfs {
    pub fn new() -> Self {
        let mut nodes = BTreeMap::new();
        nodes.insert("/".to_string(), VfsNode::Dir);
        Self { nodes }
    }

    /// Collapses `.` and `..` and joins onto `cwd` if `input` isn't already
    /// absolute. Always returns an absolute, normalized path.
    pub fn resolve(cwd: &str, input: &str) -> String {
        let joined = if input.starts_with('/') {
            input.to_string()
        } else {
            format!("{cwd}/{input}")
        };

        let mut stack: Vec<&str> = Vec::new();
        for part in joined.split('/') {
            match part {
                "" | "." => {}
                ".." => {
                    stack.pop();
                }
                p => stack.push(p),
            }
        }
        format!("/{}", stack.join("/"))
    }

    fn parent_of(path: &str) -> String {
        if path == "/" {
            return "/".to_string();
        }
        match path.rfind('/') {
            Some(0) => "/".to_string(),
            Some(i) => path[..i].to_string(),
            None => "/".to_string(),
        }
    }

    pub fn exists(&self, path: &str) -> bool {
        self.nodes.contains_key(path)
    }

    pub fn is_dir(&self, path: &str) -> bool {
        matches!(self.nodes.get(path), Some(VfsNode::Dir))
    }

    pub fn mkdir(&mut self, path: &str) -> Result<(), &'static str> {
        if self.nodes.contains_key(path) {
            return Err("already exists");
        }
        if !self.is_dir(&Self::parent_of(path)) {
            return Err("parent directory does not exist");
        }
        self.nodes.insert(path.to_string(), VfsNode::Dir);
        Ok(())
    }

    pub fn write_file(&mut self, path: &str, content: impl Into<String>) -> Result<(), &'static str> {
        if self.is_dir(path) {
            return Err("is a directory");
        }
        if !self.is_dir(&Self::parent_of(path)) {
            return Err("parent directory does not exist");
        }
        self.nodes.insert(path.to_string(), VfsNode::File(content.into()));
        Ok(())
    }

    pub fn read_file(&self, path: &str) -> Result<&str, &'static str> {
        match self.nodes.get(path) {
            Some(VfsNode::File(content)) => Ok(content),
            Some(VfsNode::Dir) => Err("is a directory"),
            None => Err("no such file or directory"),
        }
    }

    /// Direct children of `path`, as `(name, is_dir)`, in sorted order.
    pub fn list_dir(&self, path: &str) -> Result<Vec<(String, bool)>, &'static str> {
        if !self.is_dir(path) {
            return Err("not a directory");
        }

        let prefix = if path == "/" {
            "/".to_string()
        } else {
            format!("{path}/")
        };

        let mut out = Vec::new();
        for (key, node) in self.nodes.range(prefix.clone()..) {
            let Some(rest) = key.strip_prefix(&prefix) else {
                break;
            };
            if rest.is_empty() || rest.contains('/') {
                continue;
            }
            out.push((rest.to_string(), matches!(node, VfsNode::Dir)));
        }
        Ok(out)
    }

    pub fn remove(&mut self, path: &str) -> Result<(), &'static str> {
        if path == "/" {
            return Err("cannot remove root");
        }
        if !self.nodes.contains_key(path) {
            return Err("no such file or directory");
        }
        if self.is_dir(path) && !self.list_dir(path).unwrap_or_default().is_empty() {
            return Err("directory not empty");
        }
        self.nodes.remove(path);
        Ok(())
    }

    /// Soft-deletes a file into `/Trash` (overwriting any previous file of
    /// the same name that's already there). Directories fall back to a
    /// hard [`Self::remove`] — trash only handles single files for now.
    pub fn trash(&mut self, path: &str) -> Result<(), &'static str> {
        if self.is_dir(path) {
            return self.remove(path);
        }
        let content = self.read_file(path)?.to_string();
        let name = path.rsplit('/').next().unwrap_or(path);
        let dest = format!("/Trash/{name}");
        self.write_file(&dest, content)?;
        self.remove(path)
    }

    /// Every path in the filesystem, as `(path, is_dir)`, in sorted order.
    /// Used by Spotlight to search across everything on disk.
    pub fn all_paths(&self) -> impl Iterator<Item = (&str, bool)> {
        self.nodes
            .iter()
            .map(|(path, node)| (path.as_str(), matches!(node, VfsNode::Dir)))
    }
}
