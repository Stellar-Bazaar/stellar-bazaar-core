use crate::health::HealthChecker;
use crate::http::{HttpResponse, HttpStatus};
use crate::metrics::PrometheusExporter;
use crate::state::IndexerDaemonState;
use crate::BazaarRepository;

/// REST controller handling inbound HTTP requests against indexer daemon
pub struct ApiController;

impl ApiController {
    /// GET /health - Returns health check status and telemetry
    pub fn handle_health(state: &IndexerDaemonState, consecutive_errors: u32) -> HttpResponse {
        let health = HealthChecker::evaluate(state, consecutive_errors);
        HttpResponse::json(HttpStatus::Ok, &health).unwrap_or_else(|e| {
            HttpResponse::text(HttpStatus::InternalServerError, &e.to_string())
        })
    }

    /// GET /metrics - Returns Prometheus text format metrics
    pub fn handle_metrics(state: &IndexerDaemonState, consecutive_errors: u32) -> HttpResponse {
        let metrics = PrometheusExporter::render(state, consecutive_errors);
        HttpResponse::text(HttpStatus::Ok, &metrics)
    }

    /// GET /api/v1/circles - Returns list of active demand circles
    pub fn handle_list_circles(repo: &dyn BazaarRepository) -> HttpResponse {
        match repo.list_active_circles() {
            Ok(circles) => HttpResponse::json(HttpStatus::Ok, &circles).unwrap_or_else(|e| {
                HttpResponse::text(HttpStatus::InternalServerError, &e.to_string())
            }),
            Err(e) => HttpResponse::text(HttpStatus::InternalServerError, &e.to_string()),
        }
    }

    /// GET /api/v1/circles/:id - Returns details for a single demand circle
    pub fn handle_get_circle(repo: &dyn BazaarRepository, id: &str) -> HttpResponse {
        match repo.get_circle(id) {
            Ok(Some(circle)) => HttpResponse::json(HttpStatus::Ok, &circle).unwrap_or_else(|e| {
                HttpResponse::text(HttpStatus::InternalServerError, &e.to_string())
            }),
            Ok(None) => HttpResponse::not_found(&format!("Demand circle '{}' not found", id)),
            Err(e) => HttpResponse::text(HttpStatus::InternalServerError, &e.to_string()),
        }
    }

    /// GET /api/v1/circles/:id/commitments - Returns all buyer commitments for a circle
    pub fn handle_get_commitments(repo: &dyn BazaarRepository, id: &str) -> HttpResponse {
        match repo.get_commitments_for_circle(id) {
            Ok(commitments) => HttpResponse::json(HttpStatus::Ok, &commitments).unwrap_or_else(|e| {
                HttpResponse::text(HttpStatus::InternalServerError, &e.to_string())
            }),
            Err(e) => HttpResponse::text(HttpStatus::InternalServerError, &e.to_string()),
        }
    }

    /// GET /api/v1/circles/:id/offers - Returns all seller proposals for a circle
    pub fn handle_get_offers(repo: &dyn BazaarRepository, id: &str) -> HttpResponse {
        match repo.get_offers_for_circle(id) {
            Ok(offers) => HttpResponse::json(HttpStatus::Ok, &offers).unwrap_or_else(|e| {
                HttpResponse::text(HttpStatus::InternalServerError, &e.to_string())
            }),
            Err(e) => HttpResponse::text(HttpStatus::InternalServerError, &e.to_string()),
        }
    }

    /// GET /api/v1/circles/:id/disputes - Returns all dispute records for a circle
    pub fn handle_get_disputes(repo: &dyn BazaarRepository, id: &str) -> HttpResponse {
        match repo.get_disputes_for_circle(id) {
            Ok(disputes) => HttpResponse::json(HttpStatus::Ok, &disputes).unwrap_or_else(|e| {
                HttpResponse::text(HttpStatus::InternalServerError, &e.to_string())
            }),
            Err(e) => HttpResponse::text(HttpStatus::InternalServerError, &e.to_string()),
        }
    }

    /// GET /api/v1/sellers/top - Returns top-performing sellers ranked by completed deals
    pub fn handle_list_top_sellers(repo: &dyn BazaarRepository, limit: usize) -> HttpResponse {
        match repo.list_top_sellers(limit) {
            Ok(sellers) => HttpResponse::json(HttpStatus::Ok, &sellers).unwrap_or_else(|e| {
                HttpResponse::text(HttpStatus::InternalServerError, &e.to_string())
            }),
            Err(e) => HttpResponse::text(HttpStatus::InternalServerError, &e.to_string()),
        }
    }
}
