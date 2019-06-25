// Copyright 2019 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
