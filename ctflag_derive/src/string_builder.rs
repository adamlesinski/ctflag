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

pub(super) struct StringBuilder(String);

impl StringBuilder {
    pub(super) fn new() -> Self {
        StringBuilder(String::new())
    }

    pub(super) fn append<T: ToString>(&mut self, buf: T) {
        self.0.push_str(&buf.to_string());
    }
}

impl ToString for StringBuilder {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<StringBuilder> for String {
    fn from(sb: StringBuilder) -> String {
        sb.0
    }
}

impl Default for StringBuilder {
    fn default() -> Self {
        StringBuilder::new()
    }
}
