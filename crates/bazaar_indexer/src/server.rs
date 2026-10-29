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
            ApiRoute::GetCircleOffers { id } => {
                ApiController::handle_get_offers(&**repo_guard, &id)
            }
            ApiRoute::GetCircleDisputes { id } => {
                ApiController::handle_get_disputes(&**repo_guard, &id)
            }
            ApiRoute::ListTopSellers => ApiController::handle_list_top_sellers(&**repo_guard, 20),
            ApiRoute::GetSellerReputation { seller } => {
                ApiController::handle_get_seller_reputation(&**repo_guard, &seller)
            }
            ApiRoute::GetCheckpoint => ApiController::handle_get_checkpoint(&**repo_guard),
            ApiRoute::OptionsPreflight => {
                let mut resp = HttpResponse::new(
                    HttpStatus::NoContent,
                    crate::http::ContentType::TextPlain,
                    Vec::new(),
                );
                resp.headers.push((
                    "Access-Control-Allow-Methods".to_string(),
                    "GET, HEAD, OPTIONS".to_string(),
                ));
                resp.headers.push((
                    "Access-Control-Allow-Headers".to_string(),
                    "Content-Type, Authorization".to_string(),
                ));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::IndexerConfig;
    use crate::domain::*;
    use crate::http::HttpMethod;
    use crate::sqlite_repository::SqliteBazaarRepository;

    fn build_test_server() -> ApiServer {
        let mut repo = SqliteBazaarRepository::new_in_memory().expect("Failed in-mem sqlite");
        repo.record_circle(DemandCircleRecord {
            id: "CIRCLE_API_1".to_string(),
            title: "Bulk Coffee Beans".to_string(),
            category: "Commodities".to_string(),
            target_unit_price_xlm: "10.0".to_string(),
            min_volume: 100,
            max_volume: 5000,
            current_volume: 2500,
            status: "OPEN".to_string(),
            creator_address: "GCREATOR".to_string(),
            deadline_timestamp: 1720000000,
        })
        .unwrap();

        repo.record_commitment(CommitmentRecord {
            id: "COMMIT_1".to_string(),
            circle_id: "CIRCLE_API_1".to_string(),
            buyer_address: "GBUYER1".to_string(),
            quantity: 2500,
            committed_amount_xlm: "25000.0".to_string(),
            escrow_tx_hash: Some("HASH123".to_string()),
            status: "COMMITTED".to_string(),
        })
        .unwrap();

        repo.record_offer(SellerOfferRecord {
            id: "OFFER_1".to_string(),
            circle_id: "CIRCLE_API_1".to_string(),
            seller_address: "GSELLER1".to_string(),
            offered_unit_price_xlm: "9.5".to_string(),
            available_volume: 5000,
            lead_time_days: 7,
            status: "PROPOSED".to_string(),
        })
        .unwrap();

        repo.record_reputation(SellerReputationRecord {
            seller_address: "GSELLER1".to_string(),
            successful_deals: 15,
            total_volume_settled: 75000,
            total_amount_settled_xlm: "750000.0".to_string(),
            disputed_or_refunded_deals: 0,
            score_percentage: 100,
        })
        .unwrap();

        repo.record_dispute(DisputeRecord {
            id: "DISPUTE_1".to_string(),
            circle_id: "CIRCLE_API_1".to_string(),
            initiator_address: "GBUYER1".to_string(),
            dispute_reason: "Non-delivery of cargo".to_string(),
            status: "OPEN".to_string(),
            resolution_refund_to_buyers: None,
            opened_at_timestamp: 1720000100,
            resolved_at_timestamp: None,
        })
        .unwrap();

        repo.update_last_ledger(105).unwrap();

        let mut worker = IndexerWorker::new(IndexerConfig::local_testnet());
        worker.on_poll_success(105, 5);

        ApiServer::new(Box::new(repo), worker)
    }

    #[test]
    fn test_server_dispatch_all_routes() {
        let server = build_test_server();

        // 1. GET /health
        let req = HttpRequest {
            method: HttpMethod::Get,
            path: "/health".to_string(),
            query: None,
            headers: vec![],
            body: vec![],
        };
        let resp = server.dispatch(&req);
        assert_eq!(resp.status, HttpStatus::Ok);
        let s = String::from_utf8(resp.body).unwrap();
        assert!(s.contains("\"latest_ledger\":105"));

        // 2. GET /metrics
        let req = HttpRequest {
            method: HttpMethod::Get,
            path: "/metrics".to_string(),
            query: None,
            headers: vec![],
            body: vec![],
        };
        let resp = server.dispatch(&req);
        assert_eq!(resp.status, HttpStatus::Ok);
        let s = String::from_utf8(resp.body).unwrap();
        assert!(s.contains("bazaar_indexer_latest_ledger 105"));

        // 3. GET /api/v1/circles
        let req = HttpRequest {
            method: HttpMethod::Get,
            path: "/api/v1/circles".to_string(),
            query: None,
            headers: vec![],
            body: vec![],
        };
        let resp = server.dispatch(&req);
        assert_eq!(resp.status, HttpStatus::Ok);
        let s = String::from_utf8(resp.body).unwrap();
        assert!(s.contains("CIRCLE_API_1"));

        // 4. GET /api/v1/circles/CIRCLE_API_1
        let req = HttpRequest {
            method: HttpMethod::Get,
            path: "/api/v1/circles/CIRCLE_API_1".to_string(),
            query: None,
            headers: vec![],
            body: vec![],
        };
        let resp = server.dispatch(&req);
        assert_eq!(resp.status, HttpStatus::Ok);
        let s = String::from_utf8(resp.body).unwrap();
        assert!(s.contains("Bulk Coffee Beans"));

        // 5. GET /api/v1/circles/CIRCLE_API_1/commitments
        let req = HttpRequest {
            method: HttpMethod::Get,
            path: "/api/v1/circles/CIRCLE_API_1/commitments".to_string(),
            query: None,
            headers: vec![],
            body: vec![],
        };
        let resp = server.dispatch(&req);
        assert_eq!(resp.status, HttpStatus::Ok);
        let s = String::from_utf8(resp.body).unwrap();
        assert!(s.contains("GBUYER1"));

        // 6. GET /api/v1/circles/CIRCLE_API_1/offers
        let req = HttpRequest {
            method: HttpMethod::Get,
            path: "/api/v1/circles/CIRCLE_API_1/offers".to_string(),
            query: None,
            headers: vec![],
            body: vec![],
        };
        let resp = server.dispatch(&req);
        assert_eq!(resp.status, HttpStatus::Ok);
        let s = String::from_utf8(resp.body).unwrap();
        assert!(s.contains("GSELLER1"));

        // 7. GET /api/v1/circles/CIRCLE_API_1/disputes
        let req = HttpRequest {
            method: HttpMethod::Get,
            path: "/api/v1/circles/CIRCLE_API_1/disputes".to_string(),
            query: None,
            headers: vec![],
            body: vec![],
        };
        let resp = server.dispatch(&req);
        assert_eq!(resp.status, HttpStatus::Ok);
        let s = String::from_utf8(resp.body).unwrap();
        assert!(s.contains("Non-delivery of cargo"));

        // 8. GET /api/v1/sellers/top
        let req = HttpRequest {
            method: HttpMethod::Get,
            path: "/api/v1/sellers/top".to_string(),
            query: None,
            headers: vec![],
            body: vec![],
        };
        let resp = server.dispatch(&req);
        assert_eq!(resp.status, HttpStatus::Ok);
        let s = String::from_utf8(resp.body).unwrap();
        assert!(s.contains("GSELLER1"));

        // 9. GET /api/v1/sellers/GSELLER1/reputation
        let req = HttpRequest {
            method: HttpMethod::Get,
            path: "/api/v1/sellers/GSELLER1/reputation".to_string(),
            query: None,
            headers: vec![],
            body: vec![],
        };
        let resp = server.dispatch(&req);
        assert_eq!(resp.status, HttpStatus::Ok);
        let s = String::from_utf8(resp.body).unwrap();
        assert!(s.contains("\"successful_deals\":15"));

        // 10. GET /api/v1/checkpoint
        let req = HttpRequest {
            method: HttpMethod::Get,
            path: "/api/v1/checkpoint".to_string(),
            query: None,
            headers: vec![],
            body: vec![],
        };
        let resp = server.dispatch(&req);
        assert_eq!(resp.status, HttpStatus::Ok);
        let s = String::from_utf8(resp.body).unwrap();
        assert!(s.contains("\"last_ledger\":105"));

        // 11. OPTIONS /api/v1/circles
        let req = HttpRequest {
            method: HttpMethod::Options,
            path: "/api/v1/circles".to_string(),
            query: None,
            headers: vec![],
            body: vec![],
        };
        let resp = server.dispatch(&req);
        assert_eq!(resp.status, HttpStatus::NoContent);
        assert!(resp
            .headers
            .iter()
            .any(|(k, v)| k == "Access-Control-Allow-Methods" && v.contains("GET")));

        // 12. Not found
        let req = HttpRequest {
            method: HttpMethod::Get,
            path: "/nonexistent".to_string(),
            query: None,
            headers: vec![],
            body: vec![],
        };
        let resp = server.dispatch(&req);
        assert_eq!(resp.status, HttpStatus::NotFound);
    }
}
