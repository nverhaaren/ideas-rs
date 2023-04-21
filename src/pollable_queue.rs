use std::collections::VecDeque;
use crate::pollable_iterator::{Transformer, TransformExpose};
use crate::PollableIterator;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PollableQueue<T> {
    queue: VecDeque<T>,
    closed: bool,
}

#[derive(Debug)]
pub struct PollableQueueBack<'a, T> {
    inner: &'a mut PollableQueue<T>,
}

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

    pub fn expose(&mut self) -> PollableQueueBack<T> {
        PollableQueueBack { inner: self }
    }
}

impl<'a, T> PollableQueueBack<'a, T> {
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

impl<'a, T> From<&'a mut PollableQueue<T>> for PollableQueueBack<'a, T> {
    fn from(value: &'a mut PollableQueue<T>) -> Self {
        Self { inner: value }
    }
}

impl<'a, T> Extend<T> for PollableQueueBack<'a, T> {
    fn extend<I: IntoIterator<Item=T>>(&mut self, iter: I) {
        iter.into_iter().for_each(|t| self.push(t))
    }
}

impl<T, F, X, W> Extend<T> for TransformExpose<PollableQueue<T>, F, X>
where X: Fn(&mut PollableQueue<T>) -> W, W: Extend<T> {
    fn extend<I: IntoIterator<Item=T>>(&mut self, iter: I) {
        self.expose().extend(iter)
    }
}

impl<T, F, B, X, W> Transformer<T, B> for TransformExpose<PollableQueue<T>, F, X>
where X: Fn(&mut PollableQueue<T>) -> W, W: Extend<T>, F: FnMut(Option<T>) -> Option<B> {}

#[cfg(test)]
mod test {
    use std::collections::VecDeque;
    use std::mem;
    use crate::pollable_iterator::{Transform, Transformer, TransformExpose};
    use crate::pollable_queue::{PollableQueue, PollableQueueBack};
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

    fn make_upper_extractor_impl() -> TransformExpose<
        PollableQueue<String>,
        impl FnMut(Option<String>) -> Option<String>,
        impl Fn(&mut PollableQueue<String>) -> PollableQueueBack<String>
    > {
        let queue: PollableQueue<String> = PollableQueue::new();
        let mut extractor = Extractor::new();
        queue.transform_expose(
            move |maybe_s| -> Option<String> {
                if let Some(s) = maybe_s {
                    extractor.process_many(s.chars());
                }
                extractor.contents.pop_front()
            },
            PollableQueue::expose
        )
    }

    fn make_upper_extractor() -> impl PollableIterator<Item=String> {
        make_upper_extractor_impl()
    }
}
