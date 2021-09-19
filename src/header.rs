#[derive(Debug)]
pub struct MailHeader<'a> {
    pub key: &'a [u8],
    pub value: &'a [u8],
}

impl<'a> Default for MailHeader<'a> {
    fn default() -> Self {
        Self {
            key: &[],
            value: &[],
        }
    }
}

pub trait MailHeaderMap {
    fn get_all_headers(&self, key: &[u8]) -> Vec<&MailHeader>;
    fn get_all_values(&self, key: &[u8]) -> Vec<&[u8]>;
    fn get_first_header(&self, key: &[u8]) -> Option<&MailHeader>;
    fn get_first_value(&self, key: &[u8]) -> Option<&[u8]>;
}

impl<'a> MailHeaderMap for [MailHeader<'a>] {
    fn get_all_headers(&self, key: &[u8]) -> Vec<&MailHeader> {
        let mut headers: Vec<&MailHeader> = Vec::new();
        for h in self {
            if h.key.eq_ignore_ascii_case(key) {
                headers.push(h);
            }
        }
        headers
    }

    fn get_all_values(&self, key: &[u8]) -> Vec<&[u8]> {
        let mut values = vec![];
        for h in self {
            if h.key.eq_ignore_ascii_case(key) {
                values.push(h.value);
            }
        }
        values
    }

    fn get_first_header(&self, key: &[u8]) -> Option<&MailHeader> {
        for h in self {
            if h.key.eq_ignore_ascii_case(key) {
                return Some(h);
            }
        }

        None
    }

    fn get_first_value(&self, key: &[u8]) -> Option<&[u8]> {
        for h in self {
            if h.key.eq_ignore_ascii_case(key) {
                return Some(h.value);
            }
        }

        None
    }
}
