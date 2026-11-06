use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::pkg::errors::AppError;
use crate::modules::agents::dto::StepDefinition;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PipelineConfig {
    pub name: String,
    pub concurrency: Option<ConcurrencyConfig>,
    #[serde(default)]
    pub inputs: HashMap<String, InputConfig>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub jobs: HashMap<String, JobConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConcurrencyConfig {
    pub group: String,
    #[serde(default)]
    pub cancel_in_progress: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputConfig {
    pub default: Option<String>,
    pub description: Option<String>,
    pub required: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JobConfig {
    pub name: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub needs: Vec<String>,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    pub steps: Vec<StepDefinition>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

fn default_timeout() -> u64 {
    3600
}

/// Parses pipeline config from YAML or TOML content
pub fn parse_pipeline_config(raw_content: &str) -> Result<PipelineConfig, AppError> {
    // Try YAML first
    if let Ok(config) = serde_yaml::from_str::<PipelineConfig>(raw_content) {
        return Ok(config);
    }
    // Try TOML fallback
    toml::from_str::<PipelineConfig>(raw_content)
        .map_err(|e| AppError::bad_request(format!("Failed to parse pipeline config (YAML/TOML): {}", e), "INVALID_CONFIG"))
}

/// Interpolates `${VAR_NAME}` placeholders in a string using variables map
pub fn interpolate_variables(input: &str, vars: &HashMap<String, String>) -> String {
    let mut result = input.to_string();
    for (k, v) in vars {
        let placeholder = format!("${{{}}}", k);
        result = result.replace(&placeholder, v);
        // Also support plain $VAR_NAME
        let plain = format!("${}", k);
        result = result.replace(&plain, v);
    }
    result
}

/// Evaluates DAG dependency order (Topological Sort)
pub fn topological_sort_jobs(jobs: &HashMap<String, JobConfig>) -> Result<Vec<String>, AppError> {
    let mut sorted = Vec::new();
    let mut visited: HashMap<&str, bool> = HashMap::new(); // false = visiting, true = visited

    fn visit<'a>(
        node: &'a str,
        jobs: &'a HashMap<String, JobConfig>,
        visited: &mut HashMap<&'a str, bool>,
        sorted: &mut Vec<String>,
    ) -> Result<(), AppError> {
        if let Some(&is_visited) = visited.get(node) {
            if !is_visited {
                return Err(AppError::bad_request(
                    format!("Cyclic dependency detected in pipeline job 'needs' at '{}'", node),
                    "CYCLIC_DEPENDENCY",
                ));
            }
            return Ok(());
        }

        visited.insert(node, false);

        if let Some(job) = jobs.get(node) {
            for dep in &job.needs {
                if !jobs.contains_key(dep) {
                    return Err(AppError::bad_request(
                        format!("Job '{}' depends on non-existent job '{}'", node, dep),
                        "MISSING_DEPENDENCY",
                    ));
                }
                visit(dep.as_str(), jobs, visited, sorted)?;
            }
        }

        visited.insert(node, true);
        sorted.push(node.to_string());
        Ok(())
    }

    for job_key in jobs.keys() {
        if !visited.contains_key(job_key.as_str()) {
            visit(job_key.as_str(), jobs, &mut visited, &mut sorted)?;
        }
    }

    Ok(sorted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_interpolation() {
        let mut vars = HashMap::new();
        vars.insert("APP_NAME".to_string(), "coffee-backend".to_string());
        vars.insert("PORT".to_string(), "8080".to_string());

        let template = "echo Starting ${APP_NAME} on port ${PORT}...";
        let interpolated = interpolate_variables(template, &vars);
        assert_eq!(interpolated, "echo Starting coffee-backend on port 8080...");
    }

    #[test]
    fn test_dag_topological_sort() {
        let mut jobs = HashMap::new();
        jobs.insert("deploy".to_string(), JobConfig {
            name: "Deploy".to_string(),
            tags: vec![],
            needs: vec!["build".to_string()],
            timeout_seconds: 3600,
            steps: vec![],
            env: HashMap::new(),
        });
        jobs.insert("build".to_string(), JobConfig {
            name: "Build".to_string(),
            tags: vec![],
            needs: vec!["test".to_string()],
            timeout_seconds: 3600,
            steps: vec![],
            env: HashMap::new(),
        });
        jobs.insert("test".to_string(), JobConfig {
            name: "Test".to_string(),
            tags: vec![],
            needs: vec![],
            timeout_seconds: 3600,
            steps: vec![],
            env: HashMap::new(),
        });

        let sorted = topological_sort_jobs(&jobs).unwrap();
        let test_idx = sorted.iter().position(|r| r == "test").unwrap();
        let build_idx = sorted.iter().position(|r| r == "build").unwrap();
        let deploy_idx = sorted.iter().position(|r| r == "deploy").unwrap();

        assert!(test_idx < build_idx);
        assert!(build_idx < deploy_idx);
    }

    #[test]
    fn test_dag_cycle_detection() {
        let mut jobs = HashMap::new();
        jobs.insert("job-a".to_string(), JobConfig {
            name: "A".to_string(),
            tags: vec![],
            needs: vec!["job-b".to_string()],
            timeout_seconds: 3600,
            steps: vec![],
            env: HashMap::new(),
        });
        jobs.insert("job-b".to_string(), JobConfig {
            name: "B".to_string(),
            tags: vec![],
            needs: vec!["job-a".to_string()],
            timeout_seconds: 3600,
            steps: vec![],
            env: HashMap::new(),
        });

        assert!(topological_sort_jobs(&jobs).is_err());
    }

    #[test]
    fn test_parse_yaml_pipeline() {
        let yaml_content = r#"
name: "Backend CI/CD"
concurrency:
  group: "deploy-coffee-be"
  cancel_in_progress: true
inputs:
  environment:
    default: "staging"
    description: "Target environment"
jobs:
  build:
    name: "Build and Test"
    tags: ["linux", "docker"]
    steps:
      - name: "Run Unit Tests"
        run: "cargo test"
      - name: "Build Container"
        container_build:
          image: "registry.company.com/coffee-be"
          tags: ["latest", "${CI_COMMIT_SHA}"]
"#;
        let config = parse_pipeline_config(yaml_content).expect("Valid YAML pipeline");
        assert_eq!(config.name, "Backend CI/CD");
        assert!(config.concurrency.is_some());
        let conc = config.concurrency.unwrap();
        assert_eq!(conc.group, "deploy-coffee-be");
        assert!(conc.cancel_in_progress);
        assert!(config.jobs.contains_key("build"));
    }

    #[test]
    fn test_parse_toml_pipeline() {
        let toml_content = r#"
name = "Frontend Pipeline"

[inputs.environment]
default = "production"
description = "Target deployment env"

[jobs.build]
name = "Build Assets"
tags = ["linux"]
needs = []
timeout_seconds = 600

[[jobs.build.steps]]
name = "Install and Build"
run = "echo building..."
"#;
        let config = parse_pipeline_config(toml_content).expect("Valid TOML pipeline");
        assert_eq!(config.name, "Frontend Pipeline");
        assert!(config.jobs.contains_key("build"));
    }
}


