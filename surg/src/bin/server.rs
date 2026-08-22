use axum::{
    extract::Json,
    http::{HeaderValue, StatusCode},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use surg::estimator::{DurationEstimator, FeatureAwareEstimator, Gender, SurgeryFeatures};
use tower_http::cors::{Any, CorsLayer};

#[derive(Deserialize)]
struct PredictionRequest {
    surgeries: Vec<SurgeryInput>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SurgeryInput {
    surgeon: String,
    procedure: String,
    #[allow(dead_code)]
    diagnosis: Option<String>,
    #[allow(dead_code)]
    predicted_start: Option<String>,
}

#[derive(Serialize)]
struct PredictionResponse {
    predictions: Vec<Prediction>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Prediction {
    procedure: String,
    surgeon: String,
    predicted_minutes: f64,
}

async fn health() -> &'static str {
    "ok"
}

async fn predict(
    Json(request): Json<PredictionRequest>,
) -> Result<Json<PredictionResponse>, (StatusCode, String)> {
    if request.surgeries.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "surgeries must not be empty".into(),
        ));
    }
    let mut estimator = FeatureAwareEstimator::new(42);
    let predictions = request
        .surgeries
        .into_iter()
        .map(|surgery| {
            if surgery.surgeon.trim().is_empty() || surgery.procedure.trim().is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "surgeon and procedure are required".into(),
                ));
            }
            let features = SurgeryFeatures {
                procedure_code: surgery.procedure.clone(),
                surgeon_id: surgery.surgeon.clone(),
                patient_age: 50,
                patient_gender: Gender::Other,
                estimated_start: 0.0,
            };
            Ok(Prediction {
                procedure: surgery.procedure,
                surgeon: surgery.surgeon,
                predicted_minutes: estimator.sample(&features),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(PredictionResponse { predictions }))
}

fn app(allowed_origin: HeaderValue) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/predict-duration", post(predict))
        .layer(
            CorsLayer::new()
                .allow_origin(allowed_origin)
                .allow_headers(Any)
                .allow_methods(Any),
        )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = std::env::var("SURG_ADDR").unwrap_or_else(|_| "127.0.0.1:3001".into());
    let allowed_origin = std::env::var("SURG_ALLOWED_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:5173".into())
        .parse::<HeaderValue>()?;
    let listener = tokio::net::TcpListener::bind(&address).await?;
    println!("Surg_sim API listening on http://{address}");
    axum::serve(listener, app(allowed_origin)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use tower::ServiceExt;

    fn test_app() -> Router {
        app(HeaderValue::from_static("http://localhost:5173"))
    }

    #[tokio::test]
    async fn health_contract() {
        let response = test_app()
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            to_bytes(response.into_body(), usize::MAX).await.unwrap(),
            "ok"
        );
    }

    #[tokio::test]
    async fn prediction_contract_uses_camel_case() {
        let request = Request::post("/api/predict-duration")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"surgeries":[{"surgeon":"Dr. A","procedure":"ProcC"}]}"#,
            ))
            .unwrap();
        let response = test_app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let prediction = &json["predictions"][0];
        assert_eq!(prediction["surgeon"], "Dr. A");
        assert_eq!(prediction["procedure"], "ProcC");
        assert_eq!(prediction["predictedMinutes"], 86.0);
        assert!(prediction.get("predicted_minutes").is_none());
    }
}
