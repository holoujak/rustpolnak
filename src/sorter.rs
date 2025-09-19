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
    pub active: F,
    pub direction: Direction,
}

impl<F: Copy + Eq + Hash> Sorter<F> {
    pub fn new(field: F) -> Self {
        Self {
            active: field,
            direction: Direction::Asc,
        }
    }

    /// Toggle sorting for a given field
    pub fn toggle(&mut self, field: F) {
        if field == self.active {
            self.direction = match self.direction {
                Direction::Asc => Direction::Desc,
                Direction::Desc => Direction::Asc,
            };
        } else {
            self.active = field;
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
        match (self.active, self.direction) {
            (active_field, Direction::Desc) if active_field == field => {
                cmp_fn(a, b, field).reverse()
            }
            _ => cmp_fn(a, b, field),
        }
    }
}
