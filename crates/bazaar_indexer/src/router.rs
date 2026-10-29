use crate::http::HttpMethod;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiRoute {
    Health,
    Metrics,
    ListCircles,
    GetCircle { id: String },
    GetCircleCommitments { id: String },
    GetCircleOffers { id: String },
    GetCircleDisputes { id: String },
    ListTopSellers,
    GetSellerReputation { seller: String },
    GetCheckpoint,
    OptionsPreflight,
    NotFound,
}

impl ApiRoute {
    pub fn match_request(method: &HttpMethod, path: &str) -> Self {
        if *method == HttpMethod::Options {
            return ApiRoute::OptionsPreflight;
        }

        if *method != HttpMethod::Get && *method != HttpMethod::Head {
            return ApiRoute::NotFound;
        }

        // Clean trailing slash (unless root "/")
        let path = if path.len() > 1 && path.ends_with('/') {
            &path[..path.len() - 1]
        } else {
            path
        };

        match path {
            "/health" => ApiRoute::Health,
            "/metrics" => ApiRoute::Metrics,
            "/api/v1/circles" => ApiRoute::ListCircles,
            "/api/v1/sellers/top" => ApiRoute::ListTopSellers,
            "/api/v1/checkpoint" => ApiRoute::GetCheckpoint,
            _ => {
                let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
                match segments.as_slice() {
                    // /api/v1/circles/{id}
                    ["api", "v1", "circles", id] => ApiRoute::GetCircle {
                        id: (*id).to_string(),
                    },
                    // /api/v1/circles/{id}/commitments
                    ["api", "v1", "circles", id, "commitments"] => ApiRoute::GetCircleCommitments {
                        id: (*id).to_string(),
                    },
                    // /api/v1/circles/{id}/offers
                    ["api", "v1", "circles", id, "offers"] => ApiRoute::GetCircleOffers {
                        id: (*id).to_string(),
                    },
                    // /api/v1/circles/{id}/disputes
                    ["api", "v1", "circles", id, "disputes"] => ApiRoute::GetCircleDisputes {
                        id: (*id).to_string(),
                    },
                    // /api/v1/sellers/{seller}/reputation
                    ["api", "v1", "sellers", seller, "reputation"] => {
                        ApiRoute::GetSellerReputation {
                            seller: (*seller).to_string(),
                        }
                    }
                    _ => ApiRoute::NotFound,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_path_matching() {
        assert_eq!(
            ApiRoute::match_request(&HttpMethod::Get, "/health"),
            ApiRoute::Health
        );
        assert_eq!(
            ApiRoute::match_request(&HttpMethod::Get, "/metrics"),
            ApiRoute::Metrics
        );
        assert_eq!(
            ApiRoute::match_request(&HttpMethod::Get, "/api/v1/circles/"),
            ApiRoute::ListCircles
        );
        assert_eq!(
            ApiRoute::match_request(&HttpMethod::Get, "/api/v1/circles/CIRCLE_42"),
            ApiRoute::GetCircle {
                id: "CIRCLE_42".to_string()
            }
        );
        assert_eq!(
            ApiRoute::match_request(&HttpMethod::Get, "/api/v1/circles/CIRCLE_42/commitments"),
            ApiRoute::GetCircleCommitments {
                id: "CIRCLE_42".to_string()
            }
        );
        assert_eq!(
            ApiRoute::match_request(&HttpMethod::Get, "/api/v1/circles/CIRCLE_42/offers"),
            ApiRoute::GetCircleOffers {
                id: "CIRCLE_42".to_string()
            }
        );
        assert_eq!(
            ApiRoute::match_request(&HttpMethod::Get, "/api/v1/circles/CIRCLE_42/disputes"),
            ApiRoute::GetCircleDisputes {
                id: "CIRCLE_42".to_string()
            }
        );
        assert_eq!(
            ApiRoute::match_request(&HttpMethod::Get, "/api/v1/sellers/top"),
            ApiRoute::ListTopSellers
        );
        assert_eq!(
            ApiRoute::match_request(&HttpMethod::Get, "/api/v1/sellers/GBSELLER123/reputation"),
            ApiRoute::GetSellerReputation {
                seller: "GBSELLER123".to_string()
            }
        );
        assert_eq!(
            ApiRoute::match_request(&HttpMethod::Get, "/api/v1/checkpoint"),
            ApiRoute::GetCheckpoint
        );
        assert_eq!(
            ApiRoute::match_request(&HttpMethod::Options, "/api/v1/circles"),
            ApiRoute::OptionsPreflight
        );
        assert_eq!(
            ApiRoute::match_request(&HttpMethod::Get, "/unknown/endpoint"),
            ApiRoute::NotFound
        );
    }
}
