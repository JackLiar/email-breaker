use nom::bytes::streaming::{take, take_till1};
use nom::character::complete::{line_ending, multispace0, not_line_ending};
use nom::character::is_newline;
use nom::error::{make_error, ErrorKind};
use nom::number::complete::u8;
use nom::IResult;

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

#[derive(Clone, Debug, Default)]
pub struct EmailBreaker {
    #[cfg(feature = "strict-header")]
    strict_header: bool,
    #[cfg(feature = "strict-crlf")]
    strict_crlf: bool,
}

impl EmailBreaker {
    /// Parse single email header
    pub fn parse_header<'a, 'b>(&'a self, data: &'b [u8]) -> IResult<&'b [u8], MailHeader<'b>> {
        let org_data = data;
        let (data, spaces) = multispace0(data)?;
        #[cfg(feature = "strict-header")]
        if spaces.len() > 0 && self.strict_header {
            return Err(nom::Err::Error(make_error(data, ErrorKind::Space)));
        }

        let (data, key) = take_till1(|c| c == b':')(data)?;
        let (data, _) = take(1usize)(data)?;
        let (mut data, _) = multispace0(data)?;
        let value_start = org_data.len() - data.len();

        let value: &[u8];
        loop {
            let (remain, _) = not_line_ending(data)?;
            let (remain, end) = u8(remain)?;

            if !is_newline(end) {
                // if next char is '\r', take an extra byte, and stop parsing
                let (remain, _) = take(1usize)(remain)?;
                data = remain;
                value = &org_data[value_start..org_data.len() - data.len() - 2];
                break;
            }

            // otherwise is a '\n'
            #[cfg(feature = "strict-crlf")]
            if self.strict_crlf {
                return Err(nom::Err::Error(make_error(data, ErrorKind::CrLf)));
            }

            let (_, spaces) = multispace0(remain)?;
            if spaces.len() > 0 {
                // long header, keep taking bytes
                data = remain;
                continue;
            } else {
                value = &org_data[value_start..org_data.len() - data.len() - 1];
                break;
            }
        }

        let header = MailHeader { key, value };
        Ok((data, header))
    }

    /// Parse email headers
    pub fn parse_headers<'a, 'b>(
        &'a self,
        data: &'b [u8],
    ) -> IResult<&'b [u8], Vec<MailHeader<'b>>> {
        let mut headers = vec![];
        let (mut data, header) = self.parse_header(data)?;
        headers.push(header);

        while data.len() > 0 {
            match line_ending::<_, nom::error::Error<&[u8]>>(data) {
                Ok((remain, _)) => {
                    data = remain;
                    break;
                }
                Err(_) => {}
            };

            let (remain, header) = self.parse_header(data)?;
            data = remain;
            headers.push(header);
        }

        Ok((data, headers))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(feature = "strict-header")]
    #[test]
    fn space_in_front_of_header() {
        let mut breaker = EmailBreaker::default();
        breaker.strict_header = true;
        let result = breaker.parse_header(b"  Key: Value");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let is_space_error = match err {
            nom::Err::Error(e) => match e.code {
                ErrorKind::Space => true,
                _ => false,
            },
            _ => false,
        };
        assert!(is_space_error);
    }

    #[test]
    fn multi_line_value() {
        let breaker = EmailBreaker::default();
        let (_, header) = breaker
            .parse_header(b"Key: Value1\n  Value2\n   Value3\r\n")
            .unwrap();
        assert_eq!(header.key, b"Key");
        assert_eq!(header.value, b"Value1\n  Value2\n   Value3");
    }

    #[test]
    fn headers() {
        let breaker = EmailBreaker::default();
        let (remain, headers) = breaker
            .parse_headers(
                concat!(
                    "Subject: This is a test email\r\n",
                    "Content-Type: multipart/alternative; boundary=foobar\r\n",
                    "Date: Sun, 02 Oct 2016 07:06:22 -0700 (PDT)\r\n",
                    "\r\n"
                )
                .as_bytes(),
            )
            .unwrap();

        assert_eq!(remain.len(), 0);
        assert_eq!(headers.len(), 3);
        assert_eq!(headers[0].key, b"Subject");
        assert_eq!(headers[0].value, b"This is a test email");
        assert_eq!(headers[1].key, b"Content-Type");
        assert_eq!(headers[1].value, b"multipart/alternative; boundary=foobar");
        assert_eq!(headers[2].key, b"Date");
        assert_eq!(headers[2].value, b"Sun, 02 Oct 2016 07:06:22 -0700 (PDT)");
    }
}
