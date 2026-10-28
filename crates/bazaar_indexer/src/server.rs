use crate::controller::ApiController;
use crate::http::{HttpRequest, HttpResponse, HttpStatus};
use crate::router::ApiRoute;
use crate::worker::IndexerWorker;
use crate::BazaarRepository;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

/// Embedded HTTP REST API server exposing indexer database and telemetry
pub struct ApiServer {
    pub repo: Arc<Mutex<Box<dyn BazaarRepository>>>,
    pub worker: Arc<Mutex<IndexerWorker>>,
}

impl ApiServer {
    pub fn new(repo: Box<dyn BazaarRepository>, worker: IndexerWorker) -> Self {
        Self {
            repo: Arc::new(Mutex::new(repo)),
            worker: Arc::new(Mutex::new(worker)),
        }
    }

    /// Dispatches parsed HTTP request to the corresponding controller method
    pub fn dispatch(&self, req: &HttpRequest) -> HttpResponse {
        let route = ApiRoute::match_request(&req.method, &req.path);
        let repo_guard = self.repo.lock().unwrap();
        let worker_guard = self.worker.lock().unwrap();

        match route {
            ApiRoute::Health => {
                ApiController::handle_health(&worker_guard.state, worker_guard.consecutive_errors)
            }
            ApiRoute::Metrics => {
                ApiController::handle_metrics(&worker_guard.state, worker_guard.consecutive_errors)
            }
            ApiRoute::ListCircles => ApiController::handle_list_circles(&**repo_guard),
            ApiRoute::GetCircle { id } => ApiController::handle_get_circle(&**repo_guard, &id),
            ApiRoute::GetCircleCommitments { id } => {
                ApiController::handle_get_commitments(&**repo_guard, &id)
            }
            ApiRoute::GetCircleOffers { id } => ApiController::handle_get_offers(&**repo_guard, &id),
            ApiRoute::GetCircleDisputes { id } => {
                ApiController::handle_get_disputes(&**repo_guard, &id)
            }
            ApiRoute::ListTopSellers => ApiController::handle_list_top_sellers(&**repo_guard, 20),
            ApiRoute::GetSellerReputation { seller } => {
                ApiController::handle_get_seller_reputation(&**repo_guard, &seller)
            }
            ApiRoute::GetCheckpoint => ApiController::handle_get_checkpoint(&**repo_guard),
            ApiRoute::OptionsPreflight => {
                let mut resp = HttpResponse::new(HttpStatus::NoContent, crate::http::ContentType::TextPlain, Vec::new());
                resp.headers.push(("Access-Control-Allow-Methods".to_string(), "GET, HEAD, OPTIONS".to_string()));
                resp.headers.push(("Access-Control-Allow-Headers".to_string(), "Content-Type, Authorization".to_string()));
                resp
            }
            ApiRoute::NotFound => HttpResponse::not_found("Resource endpoint not found"),
        }
    }

    /// Processes a single client TCP connection
    pub fn handle_client(&self, mut stream: TcpStream) {
        let mut buffer = [0u8; 4096];
        if let Ok(bytes_read) = stream.read(&mut buffer) {
            if bytes_read > 0 {
                if let Ok(req_str) = std::str::from_utf8(&buffer[..bytes_read]) {
                    if let Some(req) = HttpRequest::parse(req_str) {
                        let resp = self.dispatch(&req);
                        let _ = stream.write_all(&resp.to_http_bytes());
                        let _ = stream.flush();
                        return;
                    }
                }
            }
        }
        let bad_resp = HttpResponse::bad_request("Malformed HTTP request payload");
        let _ = stream.write_all(&bad_resp.to_http_bytes());
        let _ = stream.flush();
    }

    /// Starts synchronous TCP listening loop
    pub fn serve(&self, addr: &str) -> Result<(), std::io::Error> {
        let listener = TcpListener::bind(addr)?;
        println!("[ApiServer] Listening on http://{}", addr);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    self.handle_client(stream);
                }
                Err(e) => {
                    eprintln!("[ApiServer] TCP stream error: {}", e);
                }
            }
        }
        Ok(())
    }
}
