use std::path::PathBuf;

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

    /*
      Only the last top dir can have a stack, so we only need to copy it to resulting one.
      Other dirs which came before are never read.
      Sometimes all dirs are on same level, so we only need to break it one-to-one.
      Stack should be evenly deep from the beginning. We only need to find a point where path length increases.
    */
    fn explode(mut self) -> Vec<Self> {
        assert!(self.stack.len() > 1);
        let top_len = self.stack.first().unwrap().components().count();
        let mut res = if top_len != self.stack.last().unwrap().components().count() {
            // Find a breaking point
            let pos = self
                .stack
                .binary_search_by(|p| p.components().count().cmp(&(top_len + 1)))
                .unwrap();
            println!("Splitting at: {pos}");
            let stack = self.stack.split_off(pos);
            vec![Stack { stack }]
        } else {
            Vec::with_capacity(self.stack.len())
        };
        // Explode the rest one-to-one, because those items are still not traversed
        res.extend(self.stack.into_iter().map(|p| Stack { stack: vec![p] }));

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
                stack: Vec::from([self.path]),
            }],
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
    /// If there is only one stack left and it has more than one element, it will be exploded into
    /// smaller stacks to facilitate iteration.
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
