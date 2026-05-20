use super::{CorpusError, compute_sha256};
use chrono::{DateTime, Utc};
use rand::{RngExt, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for corpus generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusConfig {
    /// Random seed for deterministic generation
    pub seed: u64,
    /// Number of messages to generate
    pub count: usize,
    /// Batch size for memory-efficient generation
    pub batch_size: usize,
    /// Optional output directory for generated files
    pub output_dir: Option<String>,
    /// Whether to create train/validation/test splits
    pub create_splits: bool,
    /// Split ratios (train, validation, test) - should sum to 1.0
    pub split_ratios: Option<(f64, f64, f64)>,
}

impl Default for CorpusConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            count: 100,
            batch_size: 50,
            output_dir: None,
            create_splits: false,
            split_ratios: Some((0.7, 0.15, 0.15)),
        }
    }
}

/// Information about a template file in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    /// Relative path to the template file
    pub path: String,
    /// SHA-256 hash of the template file
    pub sha256: String,
}

/// Information about a profile file in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInfo {
    /// Relative path to the profile file
    pub path: String,
    /// SHA-256 hash of the profile file
    pub sha256: String,
}

/// Information about a generated message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageInfo {
    /// Relative path to the message file
    pub path: String,
    /// SHA-256 hash of the message content
    pub sha256: String,
    /// Message type (e.g., "ADT^A01")
    pub message_type: String,
    /// Template index used to generate this message
    pub template_index: usize,
}

/// Train/validation/test split information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CorpusSplits {
    /// Training set message paths
    pub train: Vec<String>,
    /// Validation set message paths
    pub validation: Vec<String>,
    /// Test set message paths
    pub test: Vec<String>,
}

/// Manifest for reproducible message corpus generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusManifest {
    pub version: String,
    pub tool_version: String,
    pub seed: u64,
    pub templates: Vec<TemplateInfo>,
    #[serde(default)]
    pub profiles: Vec<ProfileInfo>,
    pub messages: Vec<MessageInfo>,
    pub generated_at: DateTime<Utc>,
    #[serde(default)]
    pub splits: CorpusSplits,
}

impl CorpusManifest {
    pub fn new(seed: u64) -> Self {
        Self {
            version: "1.0.0".to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            seed,
            templates: Vec::new(),
            profiles: Vec::new(),
            messages: Vec::new(),
            generated_at: Utc::now(),
            splits: CorpusSplits::default(),
        }
    }

    pub fn add_template(&mut self, path: &str, content: &str) {
        self.templates.push(TemplateInfo {
            path: path.to_string(),
            sha256: compute_sha256(content),
        });
    }
    pub fn add_profile(&mut self, path: &str, content: &str) {
        self.profiles.push(ProfileInfo {
            path: path.to_string(),
            sha256: compute_sha256(content),
        });
    }
    pub fn add_message(
        &mut self,
        path: &str,
        content: &str,
        message_type: &str,
        template_index: usize,
    ) {
        self.messages.push(MessageInfo {
            path: path.to_string(),
            sha256: compute_sha256(content),
            message_type: message_type.to_string(),
            template_index,
        });
    }
    pub fn to_json(&self) -> Result<String, CorpusError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| CorpusError::SerializationError(e.to_string()))
    }
    pub fn from_json(json: &str) -> Result<Self, CorpusError> {
        serde_json::from_str(json).map_err(|e| CorpusError::SerializationError(e.to_string()))
    }
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
    pub fn message_type_counts(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for msg in &self.messages {
            let count = counts.entry(msg.message_type.clone()).or_insert(0usize);
            *count = count.saturating_add(1);
        }
        counts
    }
    pub fn create_splits(&mut self, ratios: (f64, f64, f64)) {
        let total = self.messages.len();
        if total == 0 {
            return;
        }
        let train_count = rounded_ratio_count(total, ratios.0);
        let remaining_after_train = total.saturating_sub(train_count);
        let val_count = rounded_ratio_count(total, ratios.1).min(remaining_after_train);
        let validation_end = train_count.saturating_add(val_count);
        let mut rng = rand::rngs::StdRng::seed_from_u64(self.seed);
        let mut indices: Vec<usize> = (0..total).collect();
        for i in (1..total).rev() {
            let j = rng.random_range(0..=i);
            indices.swap(i, j);
        }
        self.splits.train = indices
            .get(..train_count)
            .unwrap_or_default()
            .iter()
            .filter_map(|&i| self.messages.get(i).map(|m| m.path.clone()))
            .collect();
        self.splits.validation = indices
            .get(train_count..validation_end)
            .unwrap_or_default()
            .iter()
            .filter_map(|&i| self.messages.get(i).map(|m| m.path.clone()))
            .collect();
        self.splits.test = indices
            .get(validation_end..)
            .unwrap_or_default()
            .iter()
            .filter_map(|&i| self.messages.get(i).map(|m| m.path.clone()))
            .collect();
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    reason = "split ratios are configured as f64 percentages by the public API"
)]
fn rounded_ratio_count(total: usize, ratio: f64) -> usize {
    if !ratio.is_finite() || ratio <= 0.0 {
        return 0;
    }
    let total_f64 = total as f64;
    let rounded = (total_f64 * ratio).round();
    if rounded <= 0.0 {
        0
    } else if rounded >= total_f64 {
        total
    } else {
        rounded as usize
    }
}
