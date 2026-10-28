use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Options,
    Head,
    Other(String),
}

impl From<&str> for HttpMethod {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "OPTIONS" => HttpMethod::Options,
            "HEAD" => HttpMethod::Head,
            other => HttpMethod::Other(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpStatus {
    Ok = 200,
    Created = 201,
    NoContent = 204,
    BadRequest = 400,
    NotFound = 404,
    MethodNotAllowed = 405,
    InternalServerError = 500,
}

impl HttpStatus {
    pub fn code(&self) -> u16 {
        *self as u16
    }

    pub fn reason_phrase(&self) -> &'static str {
        match self {
            HttpStatus::Ok => "OK",
            HttpStatus::Created => "Created",
            HttpStatus::NoContent => "No Content",
            HttpStatus::BadRequest => "Bad Request",
            HttpStatus::NotFound => "Not Found",
            HttpStatus::MethodNotAllowed => "Method Not Allowed",
            HttpStatus::InternalServerError => "Internal Server Error",
        }
    }
}

impl fmt::Display for HttpStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.code(), self.reason_phrase())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentType {
    ApplicationJson,
    TextPlain,
    TextHtml,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::ApplicationJson => "application/json; charset=utf-8",
            ContentType::TextPlain => "text/plain; charset=utf-8",
            ContentType::TextHtml => "text/html; charset=utf-8",
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub query: Option<String>,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpRequest {
    /// Parses an HTTP/1.1 raw text request string
    pub fn parse(raw: &str) -> Option<Self> {
        let mut lines = raw.lines();
        let request_line = lines.next()?;
        let mut parts = request_line.split_whitespace();
        let method_str = parts.next()?;
        let full_path = parts.next()?;

        let method = HttpMethod::from(method_str);

        let (path, query) = if let Some(idx) = full_path.find('?') {
            let p = full_path[..idx].to_string();
            let q = full_path[idx + 1..].to_string();
            (p, Some(q))
        } else {
            (full_path.to_string(), None)
        };

        let mut headers = Vec::new();
        for line in lines {
            if line.is_empty() || line == "\r" {
                break;
            }
            if let Some(pos) = line.find(':') {
                let name = line[..pos].trim().to_lowercase();
                let value = line[pos + 1..].trim().to_string();
                headers.push((name, value));
            }
        }

        Some(HttpRequest {
            method,
            path,
            query,
            headers,
            body: Vec::new(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: HttpStatus,
    pub content_type: ContentType,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn new(status: HttpStatus, content_type: ContentType, body: Vec<u8>) -> Self {
        Self {
            status,
            content_type,
            headers: vec![
                ("Connection".to_string(), "close".to_string()),
                ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
            ],
            body,
        }
    }

    pub fn json<T: Serialize>(status: HttpStatus, data: &T) -> Result<Self, serde_json::Error> {
        let bytes = serde_json::to_vec(data)?;
        Ok(Self::new(status, ContentType::ApplicationJson, bytes))
    }

    pub fn text(status: HttpStatus, msg: &str) -> Self {
        Self::new(status, ContentType::TextPlain, msg.as_bytes().to_vec())
    }

    pub fn not_found(msg: &str) -> Self {
        #[derive(Serialize)]
        struct ErrorResp<'a> {
            error: &'a str,
            code: u16,
        }
        let data = ErrorResp { error: msg, code: 404 };
        Self::json(HttpStatus::NotFound, &data).unwrap_or_else(|_| Self::text(HttpStatus::NotFound, msg))
    }

    pub fn bad_request(msg: &str) -> Self {
        #[derive(Serialize)]
        struct ErrorResp<'a> {
            error: &'a str,
            code: u16,
        }
        let data = ErrorResp { error: msg, code: 400 };
        Self::json(HttpStatus::BadRequest, &data).unwrap_or_else(|_| Self::text(HttpStatus::BadRequest, msg))
    }

    /// Serializes the response into raw HTTP/1.1 wire bytes
    pub fn to_http_bytes(&self) -> Vec<u8> {
        let mut out = format!(
            "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n",
            self.status,
            self.content_type.as_str(),
            self.body.len()
        );

        for (k, v) in &self.headers {
            out.push_str(&format!("{}: {}\r\n", k, v));
        }
        out.push_str("\r\n");

        let mut bytes = out.into_bytes();
        bytes.extend_from_slice(&self.body);
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_http_get_request() {
        let raw = "GET /api/v1/circles?limit=10 HTTP/1.1\r\nHost: localhost:8088\r\nAccept: application/json\r\n\r\n";
        let req = HttpRequest::parse(raw).expect("Failed to parse HTTP request");

        assert_eq!(req.method, HttpMethod::Get);
        assert_eq!(req.path, "/api/v1/circles");
        assert_eq!(req.query.as_deref(), Some("limit=10"));
        assert!(req.headers.iter().any(|(k, v)| k == "host" && v == "localhost:8088"));
        assert!(req.headers.iter().any(|(k, v)| k == "accept" && v == "application/json"));
    }

    #[test]
    fn test_http_response_to_bytes() {
        #[derive(Serialize)]
        struct Payload {
            message: &'static str,
        }
        let resp = HttpResponse::json(HttpStatus::Ok, &Payload { message: "pong" }).unwrap();
        let bytes = resp.to_http_bytes();
        let s = String::from_utf8(bytes).unwrap();

        assert!(s.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(s.contains("Content-Type: application/json; charset=utf-8\r\n"));
        assert!(s.contains("Access-Control-Allow-Origin: *\r\n"));
        assert!(s.contains("{\"message\":\"pong\"}"));
    }

    #[test]
    fn test_not_found_response() {
        let resp = HttpResponse::not_found("Demand circle not found");
        assert_eq!(resp.status, HttpStatus::NotFound);
        let s = String::from_utf8(resp.body).unwrap();
        assert!(s.contains("Demand circle not found"));
        assert!(s.contains("\"code\":404"));
    }
}
