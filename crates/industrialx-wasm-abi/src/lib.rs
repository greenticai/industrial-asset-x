#[derive(Debug, Clone, PartialEq)]
pub struct ComponentInput {
    pub contract: &'static str,
    pub payload_json: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentOutput {
    pub contract: &'static str,
    pub payload_json: String,
    pub evidence_json: String,
}

pub const FEATURE_EXTRACTOR_WORLD: &str = "industrialx:feature-extractor";
pub const HEALTH_SCORER_WORLD: &str = "industrialx:health-scorer";
pub const ANOMALY_DETECTOR_WORLD: &str = "industrialx:anomaly-detector";
pub const SYNTHETIC_GENERATOR_WORLD: &str = "industrialx:synthetic-generator";
pub const MODEL_EVALUATOR_WORLD: &str = "industrialx:model-evaluator";
