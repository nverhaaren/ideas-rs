pub struct FromContextFn<T, C, F: FnMut(&mut C) -> Option<T>> {
    context: C,
    f: F,
}

impl<T, C, F: FnMut(&mut C) -> Option<T>> FromContextFn<T, C, F> {
    pub fn new(context: C, f: F) -> Self {
        Self { context, f }
    }

    pub fn into_context(self) -> C {
        self.context
    }
}

impl<T, C, F: FnMut(&mut C) -> Option<T>> Iterator for &mut FromContextFn<T, C, F> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        (self.f)(&mut self.context)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn foo() -> (usize, usize) {
        let mut x = FromContextFn {
            context: 0usize,
            f: |c| {
                let input = *c;
                *c += 1;
                Some(0..input)
            }
        };
        let result = bar(x.flatten());
        (result, x.into_context())
    }

    fn bar(seq: impl IntoIterator<Item=usize>) -> usize {
        let mut result: usize = 0;
        for x in seq.into_iter().take(5) {
            result ^= x;
        }
        result
    }

    #[test]
    fn test() {
        assert_eq!(foo(), (0, 4));
    }
}
