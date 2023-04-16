use std::path::{PathBuf, Path};

type Item = (PathBuf, u64);

pub struct DirWiz {
    path: PathBuf,
}

#[derive(Debug)]
struct Stack {
    stack: Vec<PathBuf>,
}

#[derive(Debug)]
pub struct IntoIter {
    work: Vec<Stack>,
    index: usize,
}

impl Stack {
    /// Creates a stack with a single item containing a provided path.
    fn from_path(path: PathBuf) -> Self {
        Stack { stack: vec![path] }
    }

    /// Reads the subsequent directory in the stack and returns its size, or
    /// `None` if it's empty. If directory contains other directories, they
    /// would be added to the end of the stack. Symlinks are ignored.
    fn pop(&mut self) -> Option<Item> {
        let dir = self.stack.pop()?;
        let mut sum = 0;
        for entry in dir.read_dir().expect("read directory") {
            let entry = entry.unwrap();
            let meta = entry.metadata().unwrap();
            if meta.is_dir() {
                self.stack.push(entry.path());
            }
            if meta.is_file() {
                sum += meta.len();
            }
            // Ignore symlinks
        }
        Some((dir, sum))
    }

    /// Consumes the stack and converts it into a `Vec`, each containing
    /// a single directory from the original stack. If original stack contains
    /// a partially-traversed directory then direcory's stack would be placed
    /// into a single resulting element.
    /// 
    /// # Panics
    ///
    /// Panics if the source stack contains less than two entries
    fn explode(mut self) -> Vec<Self> {
        assert!(self.stack.len() > 1);

        // SAFETY: It's safe to unwrap because vector is not empty
        let first_len = self.stack.first().unwrap().components().count();
        let last_len = self.stack.last().unwrap().components().count();
        let mut res = if first_len != last_len {
            // Find a breaking point
            let pos = match self
                .stack
                .binary_search_by(|p| p.components().count().cmp(&(first_len + 1)))
            {
                Ok(v) | Err(v) => v,
            };
            let stack = self.stack.split_off(pos);
            vec![Stack { stack }]
        } else {
            Vec::with_capacity(self.stack.len())
        };
        // Convert each path from the source to a separate stack
        res.extend(self.stack.into_iter().map(|p| Stack::from_path(p)));

        res
    }
}

impl DirWiz {
    /// Creates a builder for a recursive directory iterator starting at the
    /// file path `root`. `root` should be a directory otherwise the iterater
    /// will yield zero items.
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self { path: root.as_ref().to_path_buf() }
    }
}

impl IntoIterator for DirWiz {
    type Item = Item;
    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        IntoIter {
            work: vec![Stack::from_path(self.path)],
            index: 0,
        }
    }
}

impl Iterator for IntoIter {
    type Item = Item;

    fn next(&mut self) -> Option<Item> {
        while let Some(dir) = self.get_stack() {
            if let Some(e) = dir.pop() {
                if dir.stack.is_empty() {
                    self.remove_stack();
                }
                return Some(e);
            } else {
                self.remove_stack();
            }
        }

        None
    }
}

impl IntoIter {
    pub fn interleave(&mut self) {
        self.index += 1;
        if self.index == self.work.len() {
            self.index = 0;
        }
    }

    fn remove_stack(&mut self) {
        //println!("Removed: {index}");
        self.work.swap_remove(self.index);

        if self.work.len() == self.index {
            self.index = 0;
        }
    }

    /// Returns a mutable reference to the current stack or `None` if `work` is empty.
    /// If there is only one stack left and it has more than one element,
    /// it will be exploded into smaller stacks.
    fn get_stack(&mut self) -> Option<&mut Stack> {
        println!("Work [{}]: {:?}", self.work.len(), self.work);

        if self.work.is_empty() {
            return None;
        }

        if self.work.len() == 1 && self.work[0].stack.len() > 1 {
            let exploded = self.work.remove(0).explode();
            self.work = exploded;
            self.index = 0;
        }

        self.work.get_mut(self.index)
    }
}
