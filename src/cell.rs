pub(crate) struct Wrap<O, I> {
    pub(crate) outer: O,
    pub(crate) inner: I,
}

impl<O: Clone, I> Wrap<O, I> {
    pub(crate) fn new(outer: O, inner: I) -> Self {
        Self { outer, inner }
    }
}
