use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use serde::Serialize;

use super::{error::ServerError, routes, state::OfflineState};

const MAX_HEADER_BYTES: usize = 16 * 1024;
const MAX_BODY_BYTES: usize = 1024 * 1024;
const SOCKET_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Method {
    Get,
    Post,
}

pub(super) struct Request {
    pub(super) method: Method,
    pub(super) path: String,
    pub(super) query: Vec<(String, String)>,
    pub(super) body: Vec<u8>,
}

impl Request {
    pub(super) fn query_value(&self, name: &str) -> Option<&str> {
        self.query
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
}

pub(super) struct Response {
    status: u16,
    content_type: &'static str,
    body: Vec<u8>,
}

impl Response {
    pub(super) fn json<T: Serialize>(value: &T) -> Result<Self, ServerError> {
        Ok(Self {
            status: 200,
            content_type: "application/json",
            body: serde_json::to_vec(value)?,
        })
    }

    pub(super) fn png(bytes: &[u8]) -> Self {
        Self {
            status: 200,
            content_type: "image/png",
            body: bytes.to_vec(),
        }
    }

    pub(super) fn empty(status: u16) -> Self {
        Self {
            status,
            content_type: "text/plain; charset=utf-8",
            body: Vec::new(),
        }
    }

    pub(super) fn text(status: u16, text: &str) -> Self {
        Self {
            status,
            content_type: "text/plain; charset=utf-8",
            body: text.as_bytes().to_vec(),
        }
    }
}

pub(super) fn serve(
    listener: TcpListener,
    state: Arc<OfflineState>,
    stop: Arc<AtomicBool>,
) -> Result<(), ServerError> {
    for connection in listener.incoming() {
        if stop.load(Ordering::Acquire) {
            break;
        }

        let mut stream = match connection {
            Ok(stream) => stream,
            Err(_) if stop.load(Ordering::Acquire) => break,
            Err(source) => {
                return Err(ServerError::Http {
                    context: "accept a connection",
                    source,
                });
            }
        };
        stream
            .set_read_timeout(Some(SOCKET_TIMEOUT))
            .map_err(|source| ServerError::Http {
                context: "set the connection read timeout",
                source,
            })?;
        stream
            .set_write_timeout(Some(SOCKET_TIMEOUT))
            .map_err(|source| ServerError::Http {
                context: "set the connection write timeout",
                source,
            })?;

        if let Err(error) = handle_connection(&mut stream, &state) {
            if error.is_client_disconnect() {
                continue;
            }
            return Err(error);
        }
    }

    Ok(())
}

fn handle_connection(stream: &mut TcpStream, state: &OfflineState) -> Result<(), ServerError> {
    let request = match read_request(stream) {
        Ok(Some(request)) => request,
        Ok(None) => return Ok(()),
        Err(ServerError::BadRequest { reason }) => {
            return write_response(stream, Response::text(400, &reason));
        }
        Err(ServerError::RequestTooLarge) => {
            return write_response(stream, Response::text(413, "request too large"));
        }
        Err(error) => return Err(error),
    };

    let response = match routes::handle(state, &request) {
        Ok(response) => response,
        Err(error) => {
            write_response(stream, Response::text(500, "internal server error"))?;
            return Err(error);
        }
    };

    write_response(stream, response)
}

fn read_request(stream: &mut TcpStream) -> Result<Option<Request>, ServerError> {
    let mut data = Vec::new();
    let header_end = loop {
        let mut chunk = [0u8; 4096];
        let read = stream
            .read(&mut chunk)
            .map_err(|source| ServerError::Http {
                context: "read an HTTP request",
                source,
            })?;
        if read == 0 {
            if data.is_empty() {
                return Ok(None);
            }
            return Err(ServerError::BadRequest {
                reason: "connection closed before the request headers completed".to_owned(),
            });
        }

        data.extend_from_slice(&chunk[..read]);
        if let Some(position) = data.windows(4).position(|window| window == b"\r\n\r\n") {
            if position > MAX_HEADER_BYTES {
                return Err(ServerError::RequestTooLarge);
            }
            break position;
        }
        if data.len() > MAX_HEADER_BYTES {
            return Err(ServerError::RequestTooLarge);
        }
    };

    let headers =
        std::str::from_utf8(&data[..header_end]).map_err(|_| ServerError::BadRequest {
            reason: "request headers are not valid UTF-8".to_owned(),
        })?;
    let mut lines = headers.split("\r\n");
    let request_line = lines.next().ok_or_else(|| ServerError::BadRequest {
        reason: "missing HTTP request line".to_owned(),
    })?;
    let mut request_parts = request_line.split_whitespace();
    let method = match request_parts.next() {
        Some("GET") => Method::Get,
        Some("POST") => Method::Post,
        Some(method) => {
            return Err(ServerError::BadRequest {
                reason: format!("unsupported HTTP method {method:?}"),
            });
        }
        None => {
            return Err(ServerError::BadRequest {
                reason: "missing HTTP method".to_owned(),
            });
        }
    };
    let target = request_parts
        .next()
        .ok_or_else(|| ServerError::BadRequest {
            reason: "missing HTTP request target".to_owned(),
        })?;
    let version = request_parts
        .next()
        .ok_or_else(|| ServerError::BadRequest {
            reason: "missing HTTP version".to_owned(),
        })?;
    if request_parts.next().is_some() || !version.starts_with("HTTP/1.") {
        return Err(ServerError::BadRequest {
            reason: "malformed HTTP request line".to_owned(),
        });
    }

    let (path, query) = split_target(target)?;
    let mut content_length = 0usize;
    for line in lines {
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| ServerError::BadRequest {
                reason: format!("malformed HTTP header {line:?}"),
            })?;
        if name.eq_ignore_ascii_case("content-length") {
            content_length = value.trim().parse().map_err(|_| ServerError::BadRequest {
                reason: format!("invalid Content-Length {value:?}"),
            })?;
        }
    }
    if content_length > MAX_BODY_BYTES {
        return Err(ServerError::RequestTooLarge);
    }

    let mut body = data[header_end + 4..].to_vec();
    if body.len() > content_length {
        body.truncate(content_length);
    }
    while body.len() < content_length {
        let remaining = content_length - body.len();
        let mut chunk = vec![0u8; remaining.min(4096)];
        let read = stream
            .read(&mut chunk)
            .map_err(|source| ServerError::Http {
                context: "read an HTTP request body",
                source,
            })?;
        if read == 0 {
            return Err(ServerError::BadRequest {
                reason: "connection closed before the request body completed".to_owned(),
            });
        }
        body.extend_from_slice(&chunk[..read]);
    }

    Ok(Some(Request {
        method,
        path,
        query,
        body,
    }))
}

fn split_target(target: &str) -> Result<(String, Vec<(String, String)>), ServerError> {
    let (path, query) = match target.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (target, None),
    };
    if !path.starts_with('/') {
        return Err(ServerError::BadRequest {
            reason: format!("request target {target:?} must start with '/'"),
        });
    }

    let query = match query {
        Some(query) => query
            .split('&')
            .filter(|part| !part.is_empty())
            .map(|part| {
                let (key, value) = part.split_once('=').unwrap_or((part, ""));
                Ok((percent_decode(key, true)?, percent_decode(value, true)?))
            })
            .collect::<Result<Vec<_>, ServerError>>()?,
        None => Vec::new(),
    };

    Ok((percent_decode(path, false)?, query))
}

fn percent_decode(value: &str, plus_as_space: bool) -> Result<String, ServerError> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' => {
                if index + 2 >= bytes.len() {
                    return Err(ServerError::BadRequest {
                        reason: format!("incomplete percent escape in {value:?}"),
                    });
                }
                let high = hex_value(bytes[index + 1])?;
                let low = hex_value(bytes[index + 2])?;
                output.push(high << 4 | low);
                index += 3;
            }
            b'+' if plus_as_space => {
                output.push(b' ');
                index += 1;
            }
            byte => {
                output.push(byte);
                index += 1;
            }
        }
    }

    String::from_utf8(output).map_err(|_| ServerError::BadRequest {
        reason: format!("percent-decoded value {value:?} is not valid UTF-8"),
    })
}

fn hex_value(byte: u8) -> Result<u8, ServerError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(ServerError::BadRequest {
            reason: format!("invalid percent escape byte {:?}", char::from(byte)),
        }),
    }
}

fn write_response(stream: &mut TcpStream, response: Response) -> Result<(), ServerError> {
    let reason = status_reason(response.status);
    write!(
        stream,
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n",
        response.status,
        reason,
        response.content_type,
        response.body.len()
    )
    .map_err(|source| ServerError::Http {
        context: "write an HTTP response header",
        source,
    })?;
    if response.content_type == "image/png" {
        write!(stream, "Cache-Control: max-age=2592000, public\r\n").map_err(|source| {
            ServerError::Http {
                context: "write an HTTP response cache header",
                source,
            }
        })?;
    }
    write!(stream, "\r\n").map_err(|source| ServerError::Http {
        context: "finish an HTTP response header",
        source,
    })?;
    stream
        .write_all(&response.body)
        .and_then(|()| stream.flush())
        .map_err(|source| ServerError::Http {
            context: "write an HTTP response body",
            source,
        })
}

fn status_reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        404 => "Not Found",
        413 => "Payload Too Large",
        500 => "Internal Server Error",
        _ => "Unknown",
    }
}
