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

use crate::{FromArg, FromArgError, FromArgResult};

pub enum Arg {
    Arg(String),
    Flag(FlagStruct),
}

pub struct FlagStruct {
    pub key: String,
    pub val: Option<String>,
}

pub struct FlagIterator<T>
where
    T: Iterator<Item = String>,
{
    inner: T,
}

impl<T> FlagIterator<T>
where
    T: Iterator<Item = String>,
{
    pub fn from_args(args: T) -> Self {
        FlagIterator { inner: args }
    }

    pub fn next_arg(&mut self) -> Option<String> {
        match self.next() {
            Some(Arg::Arg(val)) => Some(val),
            _ => None,
        }
    }
}

impl<T> Iterator for FlagIterator<T>
where
    T: Iterator<Item = String>,
{
    type Item = Arg;

    fn next(&mut self) -> Option<Self::Item> {
        let arg = self.inner.next()?;
        if !arg.starts_with("-") {
            Some(Arg::Arg(arg))
        } else {
            Some(Arg::Flag(match arg.find("=") {
                Some(idx) => FlagStruct {
                    key: String::from(&arg[0..idx]),
                    val: Some(String::from(&arg[idx + 1..arg.len()])),
                },
                None => FlagStruct {
                    key: arg,
                    val: None,
                },
            }))
        }
    }
}

pub fn bool_from_arg(s: Option<&str>) -> FromArgResult<bool> {
    match s {
        Some(s) => s.parse::<bool>().map_err(|_| FromArgError::new()),
        None => Ok(true),
    }
}

pub fn option_from_arg<T: FromArg>(s: &str) -> FromArgResult<Option<T>> {
    <T as FromArg>::from_arg(s).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_iterator() {
        let mut iter = FlagIterator::from_args(
            vec![
                String::from("foo"),
                String::from("--flag=true"),
                String::from("--two"),
                String::from("true"),
            ]
            .into_iter(),
        );

        assert_matches!(iter.next(), Some(Arg::Arg(arg)), arg == "foo");
        assert_matches!(
            iter.next(),
            Some(Arg::Flag(f)),
            f.key == "--flag" && matches!(&f.val, Some(val), val == "true")
        );
        assert_matches!(
            iter.next(),
            Some(Arg::Flag(f)),
            f.key == "--two" && matches!(&f.val, None)
        );
        assert_matches!(iter.next(), Some(Arg::Arg(arg)), arg == "true");
        assert_matches!(iter.next(), None);
    }
}
