use std::str::FromStr;

use anyhow::anyhow;

pub enum HTTPMethod {
    GET,
    POST,
}


impl FromStr for HTTPMethod {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "GET" => Self::GET,
            "POST" => Self::POST,
            unsupported_method => anyhow::bail!("Unsupported method '{}'", unsupported_method)
        })
    }
}


struct RequestTarget(String);

struct HTTPVersion;

enum HTTPHeader {
    Host,
    UserAgent,
    Accept,
}

struct RequestBody;

struct HTTPRequest<'a> {
    http_method: HTTPMethod,
    request_target: RequestTarget,
    // http_version: crate::HTTPVersion,
    headers: Vec<HTTPHeader>,
    // body: crate::RequestBody,
}

impl<'a> TryFrom<&'a str> for HTTPRequest<'a> {
    type Error = anyhow::Error;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut lines = value.lines();
        let Some(status_line) = lines.next() else {
            anyhow::bail!("Status line is missing");
        };
        let (http_method, request_target) = parse_status_line(status_line)?;

        let headers = lines.map(|line| line.try_into())
            .collect::<Result<_, _>>()?;

        Ok(HTTPRequest {
            http_method,
            request_target,
            headers
        })
    }
}

fn parse_status_line(status_line: &str) -> anyhow::Result<(HTTPMethod, RequestTarget)> {
    let Some((method, status_line)) = status_line.split_once(' ') else {
        return Err(anyhow!("Malformed status line"));
    };

    let method: HTTPMethod = method.parse()?;

    let Some((target, _)) = status_line.split_once(' ') else {
        return Err(anyhow!("Malformed status line"));
    };

    let target: RequestTarget = match target {
        s => {
            if s.starts_with('/') { RequestTarget(s.to_owned()) } else {
                return Err(anyhow!("Target must be started with '/'"));
            }
        }
        _ => { return Err(anyhow!("Unexpected err")); }
    };

    Ok((method, target))
}