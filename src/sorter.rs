use std::cmp::Ordering;
use std::hash::Hash;

/// Direction of sorting
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    Asc,
    Desc,
}

#[derive(Clone)]
pub struct Sorter<F: Copy + Eq + Hash> {
    pub active: Option<F>,
    pub direction: Direction,
}

impl<F: Copy + Eq + Hash> Sorter<F> {
    pub fn new() -> Self {
        Self {
            active: None,
            direction: Direction::Asc,
        }
    }

    /// Toggle sorting for a given field
    pub fn toggle(&mut self, field: F) {
        if Some(field) == self.active {
            self.direction = match self.direction {
                Direction::Asc => Direction::Desc,
                Direction::Desc => Direction::Asc,
            };
        } else {
            self.active = Some(field);
            self.direction = Direction::Asc;
        }
    }

    /// Compare two values using a custom comparator
    pub fn cmp_by<T>(
        &self,
        a: &T,
        b: &T,
        field: F,
        cmp_fn: impl Fn(&T, &T, F) -> Ordering,
    ) -> Ordering {
        let mut ord = cmp_fn(a, b, field);
        if let Some(active_field) = self.active {
            if active_field == field {
                if let Direction::Desc = self.direction {
                    ord = ord.reverse();
                }
            }
        }
        ord
    }
}
