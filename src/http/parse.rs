use crate::http::*;
use derive_more::{Error, Display};
use nom::{branch::alt, bytes::complete::*, combinator::map_res, IResult};

fn parse_method(input: &str) -> IResult<&str, Method> {
    // Note that we are stupidly match the string twice,
    // once in nom,
    // another in the from_str() constructor.
    // FIXME: Fix this shit.
    map_res(alt((tag("GET"), tag("POST"))), |s: &str| {
        Method::from_str(s)
    })(input)
}

fn parse_uri(input: &str) -> IResult<&str, String> {
    let (input, s) = take_till(|c| c == ' ')(input)?;

    // Oh no, to_vec()...
    Ok((input, s.to_string()))
}

fn parse_version(input: &str) -> IResult<&str, Version> {
    // Note: same as in method.
    // FIXME: Fix this shit.
    map_res(alt((tag("HTTP/1.0"), tag("HTTP/1.1"))), |s: &str| {
        Version::from_str(s)
    })(input)
}

fn parse_header_line(input: &str) -> IResult<&str, (String, String)> {
    let (input, key) = take_till(|c| c == ':')(input)?;
    let (input, _) = tag(": ")(input)?;
    let (input, value) = take_till(|c| c == '\r')(input)?;
    let (input, _) = tag("\r\n")(input)?;

    // to_string() masterpiece, once again.
    Ok((input, (key.to_string(), value.to_string())))
}

fn parse_headers(mut input: &str) -> IResult<&str, HashMap<String, String>> {
    let mut map = HashMap::new();

    loop {
        match parse_header_line(input) {
            Ok((i, (k, v))) => {
                input = i;
                map.insert(k, v);
            }
            Err(_) => break,
        };
    }

    let (input, _) = tag("\r\n")(input)?;

    Ok((input, map))
}

fn parse_request_parts(input: &str) -> IResult<&str, RequestParts> {
    // Parse request line
    let (input, method) = parse_method(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, uri) = parse_uri(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, version) = parse_version(input)?;
    let (input, _) = tag("\r\n")(input)?;
    let (input, headers) = parse_headers(input)?;

    // Make sure input is exhausted.
    Ok((
        input,
        RequestParts {
            method,
            uri,
            version,
            headers,
        },
    ))
}

#[derive(Debug, Display, PartialEq, Error)]
pub struct ParseError {}

impl FromStr for RequestParts {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_request_parts(s) {
            Ok((_, r)) => Ok(r),
            Err(_) => Err(ParseError {}),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request() {
        let request = "GET /love/you/tornado HTTP/1.1\r\nCookie: Name=Value&loved_one=tornado\r\nHost: example.com\r\n\r\n";
        assert_eq!(
            RequestParts::from_str(request),
            Ok(RequestParts {
                method: Method::Get,
                uri: "/love/you/tornado".into(),
                version: Version::Http11,
                headers: HashMap::from([
                    ("Host".to_string(), "example.com".to_string()),
                    (
                        "Cookie".to_string(),
                        "Name=Value&loved_one=tornado".to_string()
                    )
                ])
            })
        );
    }
}
