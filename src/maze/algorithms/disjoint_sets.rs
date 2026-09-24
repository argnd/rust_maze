/// Union-find over indices 0..n: answers "are these two already connected?"
/// and merges groups, both in near-constant time.
pub struct DisjointSets {
    parent: Vec<usize>,
}

impl DisjointSets {
    pub fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
        }
    }

    /// The representative of `i`'s group. Mutates because it flattens the
    /// path it walked (path compression), which keeps later lookups short.
    pub fn find(&mut self, i: usize) -> usize {
        let mut root = i;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        let mut current = i;
        while current != root {
            let next = self.parent[current];
            self.parent[current] = root;
            current = next;
        }
        root
    }

    /// Joins the groups of `a` and `b`; false if they were already one.
    pub fn merge(&mut self, a: usize, b: usize) -> bool {
        let root_a = self.find(a);
        let root_b = self.find(b);
        if root_a == root_b {
            return false;
        }
        self.parent[root_a] = root_b;
        true
    }
}
