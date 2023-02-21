use serde::Deserialize;
use std::{iter, ops::Range};

#[derive(Debug, Clone, Deserialize)]
pub enum CycleDirection {
    Forward,
    Backward,
    PingPong,
}

#[derive(Debug)]
pub struct Cycle<I> {
    vec: Vec<I>,
    direction: CycleDirection,
    play_head: usize,
    is_odd: bool,
    repeat: u64,
    repeat_cycle: iter::Cycle<Range<u64>>,
}

impl<I> Cycle<I> {
    pub fn new(vec: Vec<I>, direction: CycleDirection, repeat: Option<u64>) -> Cycle<I> {
        let play_head = match direction {
            CycleDirection::Backward => vec.len() - 1,
            _ => 0,
        };

        Self {
            vec,
            direction,
            play_head,
            is_odd: false,
            repeat: repeat.unwrap_or(1),
            repeat_cycle: repeat.map_or((1..2).cycle(), |n| (1..n + 1).cycle()),
        }
    }

    pub fn update_vec(&mut self, vec: Vec<I>) {
        self.vec = vec;
        self.play_head = 0;
    }

    pub fn next(&mut self) -> &I {
        let current = self.vec.get(self.play_head).unwrap();

        // If this item should be repeated, then do so
        if self.repeat > 0 && self.repeat != self.repeat_cycle.next().unwrap() {
            return current;
        }

        let max = self.vec.len().saturating_sub(1);
        // Move the play head for the next iteration
        self.play_head = match self.direction {
            CycleDirection::Forward => {
                if self.play_head >= max {
                    0
                } else {
                    self.play_head.saturating_add(1)
                }
            }

            CycleDirection::Backward => {
                if self.play_head == 0 {
                    max
                } else {
                    self.play_head.saturating_sub(1)
                }
            }

            CycleDirection::PingPong => {
                if self.play_head == max {
                    // Reached the end: go backwards
                    self.is_odd = true;
                } else if self.play_head == 0 {
                    // Reached the beginning: go forwards
                    self.is_odd = false;
                }

                if self.is_odd {
                    // Going backwards
                    self.play_head.saturating_sub(1)
                } else {
                    // Going forward
                    self.play_head.saturating_add(1)
                }
            }
        };

        current
    }
}

#[cfg(test)]
mod tests {
    use super::{Cycle, CycleDirection};

    #[test]
    fn repeat() {
        let mut c = Cycle::new(vec![1, 2], CycleDirection::Forward, Some(3));

        assert_eq!(c.next(), &1);
        assert_eq!(c.next(), &1);
        assert_eq!(c.next(), &1);
        assert_eq!(c.next(), &2);
        assert_eq!(c.next(), &2);
        assert_eq!(c.next(), &2);
        assert_eq!(c.next(), &1);
        assert_eq!(c.next(), &1);
        assert_eq!(c.next(), &1);
    }

    #[test]
    fn repeat_backwards() {
        let mut c = Cycle::new(vec![1, 2, 3], CycleDirection::Backward, Some(2));

        assert_eq!(c.next(), &3);
        assert_eq!(c.next(), &3);
        assert_eq!(c.next(), &2);
        assert_eq!(c.next(), &2);
        assert_eq!(c.next(), &1);
        assert_eq!(c.next(), &1);
    }

    #[test]
    fn repeat_ping_pong() {
        let mut c = Cycle::new(vec![1, 2, 3], CycleDirection::PingPong, Some(2));

        assert_eq!(c.next(), &1);
        assert_eq!(c.next(), &1);
        assert_eq!(c.next(), &2);
        assert_eq!(c.next(), &2);
        assert_eq!(c.next(), &3);
        assert_eq!(c.next(), &3);
        assert_eq!(c.next(), &2);
        assert_eq!(c.next(), &2);
        assert_eq!(c.next(), &1);
        assert_eq!(c.next(), &1);
    }

    #[test]
    fn forwards() {
        let mut c = Cycle::new(vec![1, 2, 3], CycleDirection::Forward, None);

        assert_eq!(c.next(), &1);
        assert_eq!(c.next(), &2);
        assert_eq!(c.next(), &3);
        assert_eq!(c.next(), &1);
    }

    #[test]
    fn backwards() {
        let mut c = Cycle::new(vec![1, 2, 3], CycleDirection::Backward, None);

        assert_eq!(c.next(), &3);
        assert_eq!(c.next(), &2);
        assert_eq!(c.next(), &1);
        assert_eq!(c.next(), &3);
    }

    #[test]
    fn ping_pong() {
        let mut c = Cycle::new(vec![1, 2, 3], CycleDirection::PingPong, None);

        assert_eq!(c.next(), &1);
        assert_eq!(c.next(), &2);
        assert_eq!(c.next(), &3);
        assert_eq!(c.next(), &2);
        assert_eq!(c.next(), &1);
        assert_eq!(c.next(), &2);
    }
}
