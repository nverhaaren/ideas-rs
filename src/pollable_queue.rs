use std::collections::VecDeque;
use std::marker::PhantomData;
use crate::pollable_iterator::{AccessorMut, PollableTransformer, TransformExpose};
use crate::PollableIterator;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PollableQueue<T> {
    queue: VecDeque<T>,
    closed: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PollableQueueBack<T> {
    inner: PollableQueue<T>,
}

#[derive(Debug)]
pub struct PollableQueueAccessor<T> {
    _phantom: PhantomData<T>,
}

impl<T> Clone for PollableQueueAccessor<T> {
    fn clone(&self) -> Self {
        Self { _phantom: self._phantom }
    }
}

impl<T> Copy for PollableQueueAccessor<T> {}

impl<T> PollableQueue<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, t: T) {
        assert!(!self.closed);
        self.queue.push_back(t);
    }

    pub fn close(&mut self) {
        self.closed = true;
    }

    pub fn closed(&self) -> bool {
        self.closed
    }

    pub fn restrict(self) -> PollableQueueBack<T> {
        PollableQueueBack { inner: self }
    }

    pub fn create_transformer<F: FnMut(Option<T>) -> Option<B>, B>(f: F) -> impl PollableTransformer<T, B> {
        let queue = Self::new();
        queue
            .transform(f)
            .expose(|queue| queue.restrict(), PollableQueueAccessor::new())
    }
}

impl<T> PollableQueueBack<T> {
    pub fn push(&mut self, t: T) {
        self.inner.push(t);
    }

    pub fn close(&mut self) {
        self.inner.close();
    }

    pub fn closed(&self) -> bool {
        self.inner.closed()
    }
}

impl<T> PollableQueueAccessor<T> {
    fn new() -> Self {
        Self { _phantom: PhantomData }
    }
}

impl<T> AccessorMut<PollableQueue<T>, PollableQueueBack<T>> for PollableQueueAccessor<T> {
    fn access(self, r: &mut PollableQueueBack<T>) -> &mut PollableQueue<T> {
        &mut r.inner
    }
}

impl<T> Default for PollableQueue<T> {
    fn default() -> Self {
        Self { queue: Default::default(), closed: Default::default() }
    }
}

impl<T> Extend<T> for PollableQueue<T> {
    fn extend<I: IntoIterator<Item=T>>(&mut self, iter: I) {
        assert!(!self.closed);
        self.queue.extend(iter)
    }
}

impl<T> FromIterator<T> for PollableQueue<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        Self { queue: iter.into_iter().collect(), closed: false }
    }
}

impl<T> Iterator for PollableQueue<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop_front()
    }
}

impl<T> PollableIterator for PollableQueue<T> {
    fn is_done(&self) -> bool {
        self.closed() && self.queue.is_empty()
    }
}

impl<T> From<PollableQueue<T>> for PollableQueueBack<T> {
    fn from(value: PollableQueue<T>) -> Self {
        Self { inner: value }
    }
}

impl<T> Extend<T> for PollableQueueBack<T> {
    fn extend<I: IntoIterator<Item=T>>(&mut self, iter: I) {
        iter.into_iter().for_each(|t| self.push(t))
    }
}

impl<T, F, B> Extend<T> for TransformExpose<PollableQueue<T>, PollableQueueBack<T>, F, PollableQueueAccessor<T>>
where F: FnMut(Option<T>) -> Option<B> {
    fn extend<I: IntoIterator<Item=T>>(&mut self, iter: I) {
        self.restricted_mut().extend(iter)
    }
}

impl<T, F, B> PollableTransformer<T, B> for TransformExpose<PollableQueue<T>, PollableQueueBack<T>, F, PollableQueueAccessor<T>>
where F: FnMut(Option<T>) -> Option<B> {}

#[cfg(test)]
mod test {
    use std::collections::VecDeque;
    use std::mem;
    use crate::pollable_iterator::{Transform, PollableTransformer, TransformExpose};
    use crate::pollable_queue::{PollableQueue, PollableQueueAccessor, PollableQueueBack};
    use crate::PollableIterator;

    struct Extractor {
        contents: VecDeque<String>,
        working: String,
        receiving: bool,
    }

    impl Extractor {
        fn new() -> Self {
            Self { contents: Default::default(), working: Default::default(), receiving: true }
        }

        fn accept(&mut self) {
            self.contents.push_back(mem::take(&mut self.working));
        }

        fn reject(&mut self) {
            self.working.truncate(0);
            self.receiving = false;
        }

        fn process(&mut self, c: char) {
            match (self.receiving, c) {
                (false, ' ') => self.receiving = true,
                (true, ' ') => if !self.working.is_empty() { self.accept() },
                (true, _) => if c.is_ascii_uppercase() {
                    self.working.push(c);
                } else {
                    self.reject();
                }
                _ => (),
            }
        }

        fn process_many(&mut self, cs: impl IntoIterator<Item=char>) {
            cs.into_iter().for_each(|c| self.process(c))
        }

        fn pop(&mut self) -> Option<String> {
            self.contents.pop_front()
        }
    }

    fn make_upper_extractor() -> impl PollableTransformer<String, String> {
        let mut extractor = Extractor::new();
        PollableQueue::create_transformer(
            move |maybe_s: Option<String>| -> Option<String> {
                if let Some(s) = maybe_s {
                    extractor.process_many(s.chars());
                }
                extractor.contents.pop_front()
            }
        )
    }
}
