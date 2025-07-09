use std::{
    fmt::Display,
    iter::{Product, Sum},
    ops::{
        Add, AddAssign, Deref, DerefMut, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub,
        SubAssign,
    },
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Currency(i64);

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
    pub const CURRENCY_SYMBOL: char = 'ŧ';
    pub const DECIMAL_SEPARATOR: char = ',';
    pub const THOUSANDS_SEPARATOR: char = '.';
    pub const ONE_CT: Currency = Currency(1);
    pub const ONE: Currency = Currency(100);
    pub const ZERO: Currency = Currency(0);
    pub const NEGATIVE_SYMBOL: char = '-';

    #[inline]
    pub const fn new(credits: i64, cents: i64) -> Self {
        Self(credits * 100 + cents)
    }

    #[inline]
    pub const fn inner(&self) -> &i64 {
        &self.0
    }

    #[inline]
    pub const fn inner_mut(&mut self) -> &mut i64 {
        &mut self.0
    }

    /// Get ONLY the cents part
    #[inline]
    pub const fn cents(&self) -> i64 {
        self.0 % 100
    }

    /// Get ONLY the major part, without cents
    #[inline]
    pub const fn credits(&self) -> i64 {
        self.0 / 100
    }

    #[inline]
    #[must_use]
    pub const fn is_negative(&self) -> bool {
        self.0.is_negative()
    }

    pub const fn round_cents(&self) -> Self {
        let cents = self.cents();
        if cents < 50 {
            Self(self.0 - cents)
        } else {
            Self(self.0 + (100 - cents))
        }
    }

    #[inline]
    pub const fn as_float(&self) -> f64 {
        self.0 as f64 / 100.0
    }
}

impl Deref for Currency {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Currency {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<i64> for Currency {
    fn from(value: i64) -> Self {
        Currency(value)
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let creds = self.credits().abs();
        let cents = self.cents().abs();

        // Format main units with thousands separators
        let main_str = if creds == 0 {
            "0".to_string()
        } else {
            let mut result = String::new();
            let main_str = creds.to_string();

            for (i, ch) in main_str.chars().rev().enumerate() {
                if i > 0 && i % 3 == 0 {
                    result.push(Self::THOUSANDS_SEPARATOR);
                }
                result.push(ch);
            }

            result.chars().rev().collect()
        };

        // Combine everything
        write!(
            f,
            "{}{}{}{:02}{}",
            if self.is_negative() {
                Self::NEGATIVE_SYMBOL.to_string()
            } else {
                "".to_string()
            },
            main_str,
            Self::DECIMAL_SEPARATOR,
            cents,
            Self::CURRENCY_SYMBOL
        )
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

impl Mul<i64> for Currency {
    type Output = Self;

    fn mul(self, rhs: i64) -> Self::Output {
        Self(self.0 * rhs)
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

impl MulAssign<i64> for Currency {
    fn mul_assign(&mut self, rhs: i64) {
        self.0 *= rhs
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
        let mut acc = Currency::new(0, 0);
        for c in iter {
            acc += c;
        }
        acc
    }
}

impl Product for Currency {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut acc = Currency::new(0, 0);
        for c in iter {
            acc *= c;
        }
        acc
    }
}

#[cfg(test)]
mod test {
    use crate::currency::Currency;

    #[test]
    fn test_currency_display() {
        assert_eq!(Currency(100000000000).to_string(), "1.000.000.000,00ŧ");
        assert_eq!(Currency(10000000000).to_string(), "100.000.000,00ŧ");
        assert_eq!(Currency(1000000000).to_string(), "10.000.000,00ŧ");
        assert_eq!(Currency(100000000).to_string(), "1.000.000,00ŧ");
        assert_eq!(Currency(10000000).to_string(), "100.000,00ŧ");
        assert_eq!(Currency(1000000).to_string(), "10.000,00ŧ");
        assert_eq!(Currency(100000).to_string(), "1.000,00ŧ");
        assert_eq!(Currency(10000).to_string(), "100,00ŧ");
        assert_eq!(Currency(100).to_string(), "1,00ŧ");
        assert_eq!(Currency(10).to_string(), "0,10ŧ");
        assert_eq!(Currency(1).to_string(), "0,01ŧ");
        assert_eq!(Currency(0).to_string(), "0,00ŧ");
        assert_eq!(Currency(-0).to_string(), "0,00ŧ");
        assert_eq!(Currency(-1).to_string(), "-0,01ŧ");
        assert_eq!(Currency(-10).to_string(), "-0,10ŧ");
        assert_eq!(Currency(-100).to_string(), "-1,00ŧ");
        assert_eq!(Currency(-10000).to_string(), "-100,00ŧ");
        assert_eq!(Currency(-100000).to_string(), "-1.000,00ŧ");
        assert_eq!(Currency(-1000000).to_string(), "-10.000,00ŧ");
        assert_eq!(Currency(-10000000).to_string(), "-100.000,00ŧ");
        assert_eq!(Currency(-100000000).to_string(), "-1.000.000,00ŧ");
        assert_eq!(Currency(-1000000000).to_string(), "-10.000.000,00ŧ");
        assert_eq!(Currency(-10000000000).to_string(), "-100.000.000,00ŧ");
        assert_eq!(Currency(-100000000000).to_string(), "-1.000.000.000,00ŧ");

        assert_eq!(crate::currency::Currency::new(1, 50).to_string(), "1,50ŧ");
        assert_eq!(CU!(1, 50).to_string(), "1,50ŧ");
        assert_eq!(CU!(0, 50).to_string(), "0,50ŧ");
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

    #[test]
    fn test_currency_roundct() {
        assert_eq!(CU!(1, 33).round_cents(), CU!(1));
        assert_eq!(CU!(1, 49).round_cents(), CU!(1));
        assert_eq!(CU!(1, 50).round_cents(), CU!(2));
    }
}
