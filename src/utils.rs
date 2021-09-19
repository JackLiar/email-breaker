use nom::bytes::streaming::{take, take_till};
use nom::IResult;

pub fn extract_between_pair_marks(data: &[u8], left: u8, right: u8) -> IResult<&[u8], &[u8]> {
    let (data, _) = take_till(|c| c == left)(data)?;
    let (data, _) = take(1usize)(data)?;
    let (data, matched) = take_till(|c| c == right)(data)?;
    let (data, _) = take(1usize)(data)?;
    Ok((matched, data))
}

#[cfg(test)]
mod test {

    #[test]
    fn extract_between_pair_marks() {
        let data = b"<12345678@test.com>";
        let (extracted, remain) = super::extract_between_pair_marks(data, b'<', b'>').unwrap();
        assert_eq!(extracted, b"12345678@test.com");
        assert_eq!(remain.len(), 0);

        let data = b"\"12345678@test.com\"";
        let (extracted, remain) = super::extract_between_pair_marks(data, b'"', b'"').unwrap();
        assert_eq!(extracted, b"12345678@test.com");
        assert_eq!(remain.len(), 0);

        let data = b"[12345678@test.com]";
        let (extracted, remain) = super::extract_between_pair_marks(data, b'[', b']').unwrap();
        assert_eq!(extracted, b"12345678@test.com");
        assert_eq!(remain.len(), 0);
    }
}
