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
