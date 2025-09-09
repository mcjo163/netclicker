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

impl From<f64> for Bits {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct BitLog<const N: usize> {
    bit_record: [Bits; N],
    i: usize,
}

impl<const N: usize> Default for BitLog<N> {
    fn default() -> Self {
        Self {
            bit_record: [Bits::default(); N],
            i: 0,
        }
    }
}

impl<const N: usize> BitLog<N> {
    pub fn track(&mut self, val: Bits) {
        self.bit_record[self.i] = val;
        self.i = (self.i + 1) % N;
    }

    pub fn diff(&self, lookback: usize) -> Bits {
        (self.get_from_end(0).0 - self.get_from_end(lookback).0).into()
    }

    pub fn to_vec(&self) -> Vec<Bits> {
        let mut v = vec![];
        v.extend_from_slice(&self.bit_record[self.i..]);
        v.extend_from_slice(&self.bit_record[..self.i]);
        v
    }

    fn get_from_end(&self, lookback: usize) -> Bits {
        assert!(lookback < N);

        let lookback_i = if self.i > lookback {
            self.i - lookback - 1
        } else {
            N - (lookback - self.i) - 1
        };
        self.bit_record[lookback_i]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_log10() {
        assert_eq!(Bits::default().log10(), 0);
        assert_eq!(Bits::new(15.0).log10(), 1);
        assert_eq!(Bits::new(1e45).log10(), 45);
    }

    #[test]
    fn bits_display() {
        assert_eq!(&format!("{}", Bits::new(15.0)), "15b");
        assert_eq!(&format!("{}", Bits::new(4.5e3)), "4.500kb");
        assert_eq!(&format!("{}", Bits::new(6.452e17)), "645.200Pb");
        assert_eq!(&format!("{}", Bits::new(8.163e63)), "8.163kQ2b");
    }

    #[test]
    fn bitlog_lookback_i_zero() {
        let log = BitLog {
            bit_record: [Bits(0.0), Bits(1.0), Bits(2.0), Bits(3.0)],
            i: 0,
        };

        assert_eq!(log.get_from_end(0).0, 3.0);
        assert_eq!(log.get_from_end(1).0, 2.0);
        assert_eq!(log.get_from_end(2).0, 1.0);
        assert_eq!(log.get_from_end(3).0, 0.0);
    }

    #[test]
    fn bitlog_lookback_i_nonzero() {
        let log = BitLog {
            bit_record: [Bits(2.0), Bits(3.0), Bits(0.0), Bits(1.0)],
            i: 2,
        };

        assert_eq!(log.get_from_end(0).0, 3.0);
        assert_eq!(log.get_from_end(1).0, 2.0);
        assert_eq!(log.get_from_end(2).0, 1.0);
        assert_eq!(log.get_from_end(3).0, 0.0);
    }

    #[test]
    fn bitlog_to_vec() {
        let log = BitLog {
            bit_record: [Bits(2.0), Bits(3.0), Bits(0.0), Bits(1.0)],
            i: 2,
        };

        assert_eq!(
            log.to_vec().iter().map(|&Bits(v)| v).collect::<Vec<_>>(),
            vec![0.0, 1.0, 2.0, 3.0],
        );
    }
}
