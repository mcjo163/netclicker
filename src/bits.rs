#![allow(dead_code)]

use std::fmt::{self, Display};

const UNIT_SUFFIXES: [char; 10] = ['k', 'M', 'G', 'T', 'P', 'E', 'Z', 'Y', 'R', 'Q'];

#[derive(Debug, Default, Clone, Copy)]
pub struct Bits(pub f64);

impl Bits {
    pub fn new(n: f64) -> Self {
        Self(n)
    }

    fn log10(&self) -> usize {
        if self.0 == 0.0 {
            0
        } else {
            self.0.log10() as usize
        }
    }

    fn suffix(&self) -> String {
        let power = self.log10();
        let (q_count, q_overflow) = (power / 30, power % 30);
        let suffix_index = q_overflow / 3;

        let mut s = String::new();

        if suffix_index > 0 {
            s.push(UNIT_SUFFIXES[suffix_index - 1]);
        }

        if q_count > 0 {
            s.push(UNIT_SUFFIXES[UNIT_SUFFIXES.len() - 1]);
        }
        if q_count > 1 {
            s.push_str(&q_count.to_string());
        }

        s.push('b');
        s
    }

    /// The number that the raw value needs to be divided by for display purposes.
    fn divisor(&self) -> f64 {
        10.0_f64.powf((self.log10() / 3 * 3) as f64)
    }
}

impl Display for Bits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let div = self.divisor();
        if div == 1.0 {
            write!(f, "{:.0}{}", self.0, self.suffix())
        } else {
            write!(f, "{:.3}{}", self.0 / div, self.suffix())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log10() {
        assert_eq!(Bits::default().log10(), 0);
        assert_eq!(Bits::new(15.0).log10(), 1);
        assert_eq!(Bits::new(1e45).log10(), 45);
    }

    #[test]
    fn display() {
        assert_eq!(&format!("{}", Bits::new(15.0)), "15b");
        assert_eq!(&format!("{}", Bits::new(4.5e3)), "4.500kb");
        assert_eq!(&format!("{}", Bits::new(6.452e17)), "645.200Pb");
        assert_eq!(&format!("{}", Bits::new(8.163e63)), "8.163kQ2b");
    }
}
