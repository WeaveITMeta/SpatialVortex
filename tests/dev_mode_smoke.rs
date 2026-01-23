// Dev Mode smoke tests (no external services required)

use spatial_vortex as sv;
use sv::ai::integration::AIModelIntegration;
use sv::storage::cache::CacheManager;
use actix_web::HttpResponse;

#[tokio::test]
async fn dev_mode_ai_integration_is_offline() {
    std::env::set_var("DEVELOPMENT_MODE", "true");

    let ai = AIModelIntegration::new(Some("dummy-key".to_string()), None);

    // In dev mode, AI integration should report unavailable even with a key
    assert!(!ai.is_ai_available());

    // Fallback generation should still work without network
    let matrix = ai.generate_subject_matrix("Test Subject").await;
    assert!(matrix.is_ok());
}

#[tokio::test]
async fn dev_mode_cache_uses_in_memory() {
    std::env::set_var("DEVELOPMENT_MODE", "true");

    // memory:// triggers in-memory/no-op behavior
    let cache = CacheManager::new("memory://", 1).await.expect("cache init");

    // Health check should succeed without Redis running
    cache.health_check().await.expect("health ok");

    // Clear should no-op but return Ok
    cache.clear_all().await.expect("clear ok");
}

#[tokio::test]
async fn dev_mode_metrics_flag() {
    std::env::set_var("DEVELOPMENT_MODE", "true");

    let json = sv::ai::monitoring_endpoints::metrics_payload();

    assert_eq!(json["dev_mode"], true);
}
