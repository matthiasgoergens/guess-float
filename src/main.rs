#![deny(clippy::pedantic)]
use num::integer::Average;
use proptest::prelude::*;
use std::cmp::Ordering;

pub trait Guesser: Sized {
    fn guess<F>(judge: F) -> Self
    where
        F: Fn(Self) -> Ordering;
}

impl Guesser for i64 {
    /// This is your run-of-the-mill binary search on whole numbers.
    fn guess<F>(judge: F) -> Self
    where
        F: Fn(Self) -> Ordering,
    {
        let mut low = Self::MIN;
        let mut high = Self::MAX;
        loop {
            let mid = low.average_floor(&high);
            match judge(mid) {
                Ordering::Greater => low = mid + 1,
                Ordering::Equal => return mid,
                Ordering::Less => high = mid - 1,
            }
        }
    }
}

impl Guesser for f64 {
    /// Here's the magic: we can use the same binary search on floats!
    /// The nice thing is that we only use at most 64 iterations.
    /// If we went with a naive approach that gets its new midpoint as the
    /// arithmetic average of the previous bounds, we'd
    /// have to do about one thousand iterations: log(1.7976931348623155e+308, 2) = 1024.0
    /// In addition to having to worry about all kinds of extra floating point weirdness,
    /// like infinities and NaN.
    fn guess<F>(judge: F) -> Self
    where
        F: Fn(Self) -> Ordering,
    {
        #[allow(clippy::cast_sign_loss)]
        let convert = |e: i64| f64::from_bits(if e >= 0 { e } else { e ^ i64::MAX } as u64);
        convert(Guesser::guess(|e| judge(convert(e))))
    }
}

proptest! {
    #[test]
    fn guesess_int(x in any::<i64>()) {
        let judge = |e| x.cmp(&e);
        let guessed = Guesser::guess(judge);
        prop_assert_eq!(x, guessed);
    }
    #[test]
    fn guesses_float(x in prop::num::f64::ANY) {
        let judge = |e| x.total_cmp(&e);
        let guessed = Guesser::guess(judge);
        prop_assert_eq!(x.total_cmp(&guessed), Ordering::Equal);
    }
}

#[allow(clippy::float_cmp)]
fn main() {
    for x in [
        0,
        1,
        i64::MIN,
        i64::MAX,
        i64::MIN + 1,
        i64::MAX - 1,
        i64::MIN / 2,
        i64::MAX / 2,
        i64::MIN / 2 + 1,
        42,
        -1,
    ] {
        let _: &i64 = &x;
        let judge = |e| x.cmp(&e);
        let guessed = Guesser::guess(judge);
        assert_eq!(x.cmp(&guessed), Ordering::Equal);
    }

    #[allow(clippy::cast_sign_loss)]
    for x in [
        -0.0,
        1.0,
        -f64::INFINITY,
        f64::NAN,
        -f64::NAN,
        10.0 - std::f64::consts::PI,
        0.0,
        42.1,
        f64::from_bits(u64::MAX),
        f64::from_bits(u64::MIN),
        f64::from_bits(1),
        f64::from_bits(i64::MIN as u64),
        f64::from_bits(i64::MAX as u64),
    ] {
        let _: &f64 = &x;
        let judge = |e| x.total_cmp(&e);
        let guessed = Guesser::guess(judge);
        assert_eq!(x.total_cmp(&guessed), Ordering::Equal);
    }
}
