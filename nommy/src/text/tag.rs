use std::iter::FromIterator;

use crate::*;

#[derive(Debug, Copy, Clone, PartialEq)]
/// Tag is a generic type that implements Parse to match the given string exactly
///
/// ```
/// use nommy::{Parse, IntoBuf, text::Tag};
/// let mut buffer = "foobarbaz".chars().into_buf();
/// Tag::<"foobar">::parse(&mut buffer).unwrap();
/// Tag::<"baz">::parse(&mut buffer).unwrap();
/// ```
pub struct Tag<const TAG: &'static str>;

impl<const TAG: &'static str> Process for Tag<TAG> {
    type Output = Self;
    fn process(self) -> Self::Output {
        self
    }
}

impl<const TAG: &'static str> Peek<char> for Tag<TAG> {
    fn peek(input: &mut impl Buffer<char>) -> bool {
        TAG.chars().eq(input.take(TAG.len()))
    }
}

impl<const TAG: &'static str> Parse<char> for Tag<TAG> {
    fn parse(input: &mut impl Buffer<char>) -> eyre::Result<Self> {
        let s = String::from_iter(input.take(TAG.len()));
        if TAG == &s {
            Ok(Tag)
        } else {
            Err(eyre::eyre!("failed to parse tag {:?}, found {:?}", TAG, s))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse, Buffer, Parse};

    #[test]
    fn test_parse_matches() {
        let mut input = "(){}[]<>".chars().into_buf();
        Tag::<"(">::parse(&mut input).unwrap();
        Tag::<")">::parse(&mut input).unwrap();
        Tag::<"{">::parse(&mut input).unwrap();
        Tag::<"}">::parse(&mut input).unwrap();
        Tag::<"[">::parse(&mut input).unwrap();
        Tag::<"]">::parse(&mut input).unwrap();
        Tag::<"<">::parse(&mut input).unwrap();
        Tag::<">">::parse(&mut input).unwrap();
        assert!(input.next().is_none())
    }

    #[test]
    fn test_peek_matches() {
        let mut input = "(){}[]<>".chars().into_buf();
        let mut cursor = input.cursor();
        assert!(Tag::<"(">::peek(&mut cursor));
        assert!(Tag::<")">::peek(&mut cursor));
        assert!(Tag::<"{">::peek(&mut cursor));
        assert!(Tag::<"}">::peek(&mut cursor));
        assert!(Tag::<"[">::peek(&mut cursor));
        assert!(Tag::<"]">::peek(&mut cursor));
        assert!(Tag::<"<">::peek(&mut cursor));
        assert!(Tag::<">">::peek(&mut cursor));
        assert!(cursor.next().is_none())
    }

    #[test]
    fn test_parse_errors() {
        let res: Result<Tag<"(">, _> = parse("1".chars());
        assert_eq!(format!("{}", res.unwrap_err()), "failed to parse tag \"(\", found \"1\"");

        let res: Result<Tag<")">, _> = parse("1".chars());
        assert_eq!(format!("{}", res.unwrap_err()), "failed to parse tag \")\", found \"1\"");
    }
}
