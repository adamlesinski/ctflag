#![allow(unused_macros)]

macro_rules! assert_matches {
    ($actual:expr, $expected:pat, $qual:expr) => {
        assert!(match $actual {
            $expected => $qual,
            _ => false,
        })
    };

    ($actual:expr, $expected:pat) => {
        assert!(match $actual {
            $expected => true,
            _ => false,
        })
    };
}

macro_rules! matches {
    ($actual:expr, $expected:pat, $qual:expr) => {
        match $actual {
            $expected => $qual,
            _ => false,
        }
    };

    ($actual:expr, $expected:pat) => {
        match $actual {
            $expected => true,
            _ => false,
        }
    };
}
