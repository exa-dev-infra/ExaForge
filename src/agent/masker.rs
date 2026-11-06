use std::collections::HashMap;

/// Masker replaces any secret values with `***` in real time
#[derive(Debug, Clone)]
pub struct SecretMasker {
    masked_strings: Vec<String>,
}

impl SecretMasker {
    pub fn new(secrets: &HashMap<String, String>) -> Self {
        let mut masked_strings: Vec<String> = secrets
            .values()
            .filter(|v| v.len() >= 3) // Only mask secrets of reasonable length to prevent masking trivial strings
            .cloned()
            .collect();

        // Sort by length descending so longer secrets are replaced first
        masked_strings.sort_by(|a, b| b.len().cmp(&a.len()));

        Self { masked_strings }
    }

    pub fn mask(&self, line: &str) -> String {
        let mut result = line.to_string();
        for secret in &self.masked_strings {
            result = result.replace(secret, "***");
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_masking() {
        let mut secrets = HashMap::new();
        secrets.insert("DB_PASS".to_string(), "super_secret_123".to_string());
        secrets.insert("API_KEY".to_string(), "xyz987token".to_string());

        let masker = SecretMasker::new(&secrets);

        let log1 = "Connecting to database with password super_secret_123 now";
        assert_eq!(
            masker.mask(log1),
            "Connecting to database with password *** now"
        );

        let log2 = "Bearer xyz987token authenticated";
        assert_eq!(masker.mask(log2), "Bearer *** authenticated");
    }
}

