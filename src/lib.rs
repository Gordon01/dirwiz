use std::{collections::BTreeSet, path::PathBuf};

type Item = (PathBuf, u64);
pub struct DirWiz {
    path: PathBuf,
}

#[derive(Debug)]
struct Stack {
    stack: BTreeSet<PathBuf>,
}

#[derive(Debug)]
pub struct IntoIter {
    work: Vec<Stack>,
    index: usize,
}

impl Stack {
    fn pop(&mut self) -> Option<Item> {
        let dir = self.stack.pop_last()?;
        let mut sum = 0;
        for entry in dir.read_dir().expect("read directory") {
            let entry = entry.unwrap();
            let meta = entry.metadata().unwrap();
            if meta.is_dir() {
                self.stack.insert(entry.path());
            }
            if meta.is_file() {
                sum += meta.len();
            }
            // Ignore symlinks
        }
        Some((dir, sum))
    }

    fn explode(mut self) -> Vec<Self> {
        let mut res = Vec::new();

        while let Some(e) = self.stack.pop_first() {
            let mut new_stack: BTreeSet<_> = self
                .stack
                .iter()
                .filter(|p| p.starts_with(&e))
                .cloned()
                .collect();
            self.stack.retain(|p| !p.starts_with(&e));
            new_stack.insert(e);
            res.push(Stack { stack: new_stack });
        }

        res
    }
}

impl DirWiz {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl IntoIterator for DirWiz {
    type Item = Item;
    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        IntoIter {
            work: vec![Stack {
                stack: BTreeSet::from([self.path]),
            }],
            index: 0,
        }
    }
}

impl Iterator for IntoIter {
    type Item = Item;

    fn next(&mut self) -> Option<Item> {
        let index = &mut self.index;
        while !self.work.is_empty() {
            if let Some(e) = self.work[*index].pop() {
                return Some(e);
            }

            println!("Removed: {index}");
            self.work.swap_remove(*index);

            if self.work.len() == *index {
                *index = 0;
            }
            if self.work.len() == 1 && self.work[*index].stack.len() > 2 {
                let exploded = self.work.remove(*index).explode();
                let _ = std::mem::replace(&mut self.work, exploded);
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
}
