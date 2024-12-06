use std::{cmp::{max, min}, ops::Range, rc::Rc};

/// A source code location for a parsed token or node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loc {
    pub source: LocSource,
    pub pos: usize,
    pub len: usize,
}

impl Loc {
    pub fn new(source: LocSource, pos: usize, len: usize) -> Self {
        Self { source, pos, len }
    }

    /// Creates a new [Loc] spanning the entire range including two others.
    /// Panics if the underlying sources aren't the same.
    pub fn new_spanning(a: &Loc, b: &Loc) -> Self {
        if a.source != b.source {
            panic!("`new_spanning` loc sources are not equal");
        }
        let source = a.source.clone();

        // Find lowest start and highest end, then build a loc out of that
        let a_range = a.range();
        let b_range = b.range();
        let start = min(a_range.start, b_range.start);
        let end = max(a_range.end, b_range.end);
        Loc::new(source, start, end - start + 1)
    }
    
    /// The contents of the source range highlighted by ths [Loc].
    pub fn contents(&self) -> String {
        self.source.contents[self.range()].to_owned()
    }

    /// The source range of characters covered by this [Loc].
    pub fn range(&self) -> Range<usize> {
        self.pos..(self.pos + self.len)
    }

    /// Temporary method to generate a meaningless [Loc].
    /// Should exist when everything else is done!
    pub fn stub() -> Self {
        Self::new(
            LocSource::new("(stub)".to_owned(), Rc::new(String::new())),
            0, 0
        )
    }
}

/// A source that a [Loc] can refer to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocSource {
    pub name: String,
    pub contents: Rc<String>,
}

impl LocSource {
    pub fn new(name: String, contents: Rc<String>) -> Self {
        Self { name, contents }
    }
}
