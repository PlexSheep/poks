use std::{
    fmt::Display,
    iter::{Product, Sum},
    ops::{
        Add, AddAssign, Deref, DerefMut, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub,
        SubAssign,
    },
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Currency(u64);

#[macro_export]
macro_rules! CU {
    ($cr:tt) => {
        $crate::currency::Currency::new($cr, 0)
    };
    ($cr:tt,$ct:tt) => {
        $crate::currency::Currency::new($cr, $ct)
    };
}

impl Currency {
    const SIGN: char = 'ŧ';
    const DECIMAL_SEPARATOR: char = ',';
    const VISUAL_SEPARATOR: char = '.';
    const ONE_CT: Currency = Currency(1);
    const ONE: Currency = Currency(100);
    const ZERO: Currency = Currency(0);

    pub fn new(credits: u64, cents: u64) -> Self {
        Self(credits * 100 + cents)
    }

    pub fn inner(&self) -> &u64 {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut u64 {
        &mut self.0
    }

    /// Get ONLY the cents part
    pub fn cents(&self) -> u64 {
        self.0 % 100
    }

    /// Get ONLY the major part, without cents
    pub fn credits(&self) -> u64 {
        self.round_cents().0 / 100
    }

    pub fn round_cents(&self) -> Self {
        let cents = self.cents();
        if cents < 50 {
            Self(self.0 - cents)
        } else {
            Self(self.0 + (100 - cents))
        }
    }

    pub fn as_float(&self) -> f64 {
        self.0 as f64 / 100.0
    }
}

impl Deref for Currency {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Currency {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<u64> for Currency {
    fn from(value: u64) -> Self {
        Currency(value)
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::new();
        let credits_s = self.credits().to_string();
        let mut it = credits_s.chars().into_iter().rev().enumerate().peekable();
        for (index, decimal) in it.clone() {
            buf.push(decimal);
            if index % 2 == 0 && it.peek().is_some() {
                buf.push(Self::VISUAL_SEPARATOR);
            }
        }
        let mut buf: String = buf.chars().into_iter().rev().collect();
        buf.push(Self::DECIMAL_SEPARATOR);
        buf.push_str(&self.cents().to_string());
        buf.push(Self::SIGN);

        write!(f, "{buf}")
    }
}

impl Add for Currency {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl Sub for Currency {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}
impl Mul for Currency {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Div for Currency {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Rem for Currency {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}

impl AddAssign for Currency {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl SubAssign for Currency {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0
    }
}

impl MulAssign for Currency {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0
    }
}

impl DivAssign for Currency {
    fn div_assign(&mut self, rhs: Self) {
        self.0 /= rhs.0
    }
}

impl RemAssign for Currency {
    fn rem_assign(&mut self, rhs: Self) {
        self.0 %= rhs.0
    }
}

impl Sum for Currency {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.map(|c| c).sum()
    }
}

impl Product for Currency {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.map(|c| c).product()
    }
}

#[cfg(test)]
mod test {
    use crate::currency::Currency;

    #[test]
    fn test_currency_display() {
        assert_eq!(Currency(10000000000).to_string(), "10.000.000,00ŧ");
        assert_eq!(Currency(1000000000).to_string(), "1.000.000,00ŧ");
        assert_eq!(Currency(100000000).to_string(), "100.000,00ŧ");
        assert_eq!(Currency(10000000).to_string(), "10.000,00ŧ");
        assert_eq!(Currency(1000000).to_string(), "10.000,00ŧ");
        assert_eq!(Currency(100000).to_string(), "1.000,00ŧ");
        assert_eq!(Currency(10000).to_string(), "100,00ŧ");
        assert_eq!(Currency(100).to_string(), "1,00ŧ");
        assert_eq!(Currency(10).to_string(), "0,10ŧ");
        assert_eq!(Currency(1).to_string(), "0,01ŧ");
        assert_eq!(Currency(0).to_string(), "0,00ŧ");
    }

    #[test]
    fn test_currency_calc() {
        assert_eq!(Currency(1) + Currency(99), Currency(100));
        assert_eq!(Currency(100) - Currency(1), Currency(99));
        assert_eq!(Currency(2) * Currency(99), Currency(198));
        assert_eq!(Currency(33) / Currency(11), Currency(3));
        assert_eq!(Currency(33) % Currency(11), Currency(0));
        assert_eq!(Currency(33) % Currency(10), Currency(3));
    }
}
