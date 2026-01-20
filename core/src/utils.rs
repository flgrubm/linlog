// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

pub(crate) struct SlidingWindow<I: Iterator> {
    iter: I,
    previous: I::Item,
    final_value: I::Item,
    is_done: bool,
}

impl<I> Iterator for SlidingWindow<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = (I::Item, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done {
            None
        } else if let Some(next_value) = self.iter.next() {
            let current_value = self.previous.clone();
            self.previous = next_value.clone();
            Some((current_value, next_value))
        } else {
            self.is_done = true;
            Some((self.previous.clone(), self.final_value.clone()))
        }
    }
}

pub(crate) trait SlidingIterator: Iterator {
    fn slide_iter(self, final_value: Self::Item) -> SlidingWindow<Self>
    where
        Self: Sized,
        Self::Item: Clone,
    {
        let mut iter = self;

        let previous: Self::Item;
        let is_done: bool;
        if let Some(first) = iter.next() {
            previous = first;
            is_done = false;
        } else {
            previous = final_value.clone();
            is_done = true;
        }

        SlidingWindow {
            iter,
            previous,
            final_value,
            is_done,
        }
    }
}

impl<I: Iterator> SlidingIterator for I {}
