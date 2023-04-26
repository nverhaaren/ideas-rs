use std::collections::VecDeque;
use crate::pollable_iterator::{PollableTransformer, Transform};
use crate::PollableIterator;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PollableQueue<T> {
    queue: VecDeque<T>,
    closed: bool,
}

pub struct ConsumingIter<'a, T, F> {
    transform: &'a mut Transform<PollableQueue<T>, F>,
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

impl<'a, T, F, B> Iterator for ConsumingIter<'a, T, F>
where F: FnMut(Option<T>) -> Option<B> {
    type Item = B;
    fn next(&mut self) -> Option<Self::Item> {
        self.transform.next()
    }
}

impl<'a, T, F, B> PollableIterator for ConsumingIter<'a, T, F>
where F: FnMut(Option<T>) -> Option<B> {
    fn is_done(&self) -> bool {
        self.transform.is_done()
    }
}

impl<T, F> Extend<T> for Transform<PollableQueue<T>, F> {
    fn extend<I: IntoIterator<Item=T>>(&mut self, iter: I) {
        self.it.extend(iter)
    }
}

impl<T, F, B> PollableTransformer<T, B> for Transform<PollableQueue<T>, F>
where F: FnMut(Option<T>) -> Option<B> {
    fn close(&mut self) {
        self.it.close()
    }
}

#[cfg(test)]
mod test {
    use std::collections::VecDeque;
    use std::mem;
    use crate::pollable_iterator::PollableTransformer;
    use crate::pollable_queue::{PollableQueue};
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

        fn finalize(&mut self) {
            self.process(' ');
            self.receiving = false;
        }

        fn pop(&mut self) -> Option<String> {
            self.contents.pop_front()
        }
    }

    #[test]
    fn test_extractor() {
        let mut extractor = Extractor::new();
        extractor.process('H');
        assert!(extractor.pop().is_none());
        extractor.process('I');
        extractor.finalize();
        assert_eq!(extractor.pop(), Some(String::from("HI")))
    }

    fn make_upper_extractor<'a>() -> impl PollableTransformer<&'a str, String> {
        let mut extractor = Extractor::new();
        PollableQueue::new().transform_using(
            move |maybe_s: Option<&'a str>| -> Option<String> {
                if let Some(s) = maybe_s {
                    extractor.process_many(s.chars());
                } else {
                    extractor.finalize();
                }
                extractor.contents.pop_front()
            }
        )
    }

    #[test]
    fn test_upper_extractor() {
        let n = 7;
        let rest: Vec<_> = (0..n).map(|i| format!("{i}")).collect();
        let mut extractor = make_upper_extractor();
        let mut caps = vec![];
        extractor.extend(["  Fo", "oBA", "R; HEL", "L", "O  Wurld WORLD", "  !!"]);
        caps.extend(extractor.by_ref());
        extractor.extend(rest.iter().map(|s| s.as_str()));
        caps.extend(extractor.by_ref());
        extractor.close();
        caps.extend(extractor);
        assert_eq!(caps, vec![String::from("HELLO"), String::from("WORLD")]);
    }

    #[test]
    fn test_end() {
        let mut extractor = make_upper_extractor();
        extractor.feed("HI");
        extractor.close();
        let result: Vec<_> = extractor.collect();
        assert_eq!(result, [String::from("HI")]);
    }

    #[test]
    fn test_transform_iter() {
        let mut extractor = make_upper_extractor();
        let mut caps = vec![];
        caps.extend(extractor.transform_iter(["  Fo", "oBA", "R; HEL"]));
        assert!(caps.is_empty());
        caps.extend(extractor.transform_iter([ "L", "O  Wurld WORLD"]));
        assert_eq!(caps, vec![String::from("HELLO")]);
        extractor.feed(" !!");
        extractor.close();
        caps.extend(extractor);
        assert_eq!(caps, vec![String::from("HELLO"), String::from("WORLD")]);
    }
}
