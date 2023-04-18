pub trait PollableIterator: Iterator {
    fn is_done(&self) -> bool;

    fn transform<B, F>(self, f: F) -> Transform<Self, F>
    where F: FnMut(Option<Self::Item>) -> Option<B>, Self: Sized {
        Transform { it: self, f, done: false }
    }
}

pub struct Transform<I, F> {
    it: I,
    f: F,
    done: bool,
}

impl<I: PollableIterator, B, F: FnMut(Option<I::Item>) -> Option<B>> Iterator for Transform<I, F> {
    type Item = B;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        if self.it.is_done() {
            let result = (self.f)(None);
            if result.is_none() {
                self.done = true;
            }
            result
        } else {
            (self.f)(Some(self.it.next()?))
        }
    }
}

impl<I: PollableIterator, B, F: FnMut(Option<I::Item>) -> Option<B>> PollableIterator for Transform<I, F> {
    fn is_done(&self) -> bool {
        self.done
    }
}
