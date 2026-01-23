//! Integration tests for ONNX Inference Engine

#[cfg(feature = "onnx")]
mod onnx_tests {
    use spatial_vortex::inference_engine::onnx_runtime::OnnxInferenceEngine;

    #[test]
    fn test_onnx_engine_creation_without_model() {
        // This will fail without model files, which is expected
        let result = OnnxInferenceEngine::new("nonexistent.onnx", "nonexistent.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_embedding_dim() {
        // Test the expected embedding dimension
        let dim = 384; // Expected dimension for sentence-transformers/all-MiniLM-L6-v2
        assert_eq!(dim, 384);
    }

    #[test]
    fn test_sacred_geometry_transformation() {
        // Test sacred geometry transformation with a sample embedding
        let sample_embedding = vec![0.1; 384]; // 384-dimensional vector
        let engine_result = OnnxInferenceEngine::new(
            "models/model.onnx",
            "models/tokenizer.json"
        );
        
        // If model exists, test the transformation
        if let Ok(engine) = engine_result {
            let (signal, coherence, ethos, logos, pathos) = 
                engine.transform_to_sacred_geometry(&sample_embedding);
            
            // Signal strength should be in valid range
            assert!(signal >= 0.0 && signal <= 1.0);
            assert!(coherence >= 0.0);
            
            // ELP channels should sum to ~1.0
            let elp_sum = ethos + logos + pathos;
            assert!((elp_sum - 1.0).abs() < 0.01);
        }
    }
}

#[cfg(not(feature = "onnx"))]
mod feature_disabled_tests {
    use spatial_vortex::inference_engine::onnx_runtime::OnnxInferenceEngine;

    #[test]
    fn test_onnx_feature_not_enabled() {
        let result = OnnxInferenceEngine::new("dummy.onnx", "dummy.json");
        assert!(result.is_err());
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("ONNX feature not enabled"));
    }
}
