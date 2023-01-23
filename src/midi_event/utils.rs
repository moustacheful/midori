use std::cmp;

use nom::{
  error::{make_error, ErrorKind, ParseError},
  Err, IResult, Needed,
};

/// Compares two slices to check for equality
pub fn compare_u8_slices(a: &[u8], b: &[u8]) -> cmp::Ordering {
  a.iter()
    .zip(b)
    .map(|(x, y)| x.cmp(y))
    .find(|&ord| ord != cmp::Ordering::Equal)
    .unwrap_or(a.len().cmp(&b.len()))
}

/// Similar to `be_u8` from `nom`, but checks the most significant bit is 0
pub fn be_u7<'a, E>(i: &'a [u8]) -> IResult<&'a [u8], u8, E>
where
  E: ParseError<&'a [u8]>,
{
  if i.is_empty() {
    Result::Err(Err::Incomplete(Needed::new(1)))
  } else {
    let val = i[0];
    if val > 127 {
      Err(Err::Error(make_error(i, ErrorKind::TooLarge)))
    } else {
      Ok((&i[1..], val))
    }
  }
}
