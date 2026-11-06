use std::collections::HashMap;
use uuid::Uuid;

use exaforge::{
    agent::masker::SecretMasker,
    modules::{
        agents::dto::{
            AgentToControllerMessage, ContainerBuildConfig, ControllerToAgentMessage,
            StepDefinition,
        },
        pipelines::parser::{interpolate_variables, parse_pipeline_config, topological_sort_jobs},
    },
    pkg::crypto::{decrypt_secret, encrypt_secret, generate_agent_token, verify_hmac_sha256},
};

#[test]
fn test_agent_protocol_json_serialization() {
    let job_id = Uuid::new_v4();
    let pipeline_id = Uuid::new_v4();

    // 1. Controller to Agent: AssignJob
    let mut secrets = HashMap::new();
    secrets.insert("DATABASE_URL".to_string(), "postgres://secret:pass@db:5432/prod".to_string());
    secrets.insert("API_KEY".to_string(), "sec_live_998877".to_string());

    let mut env = HashMap::new();
    env.insert("CI".to_string(), "true".to_string());
    env.insert("ENVIRONMENT".to_string(), "production".to_string());

    let assign_msg = ControllerToAgentMessage::AssignJob {
        job_id,
        pipeline_id,
        repo_url: "https://github.com/my-org/coffee.git".to_string(),
        git_ref: "refs/heads/main".to_string(),
        git_commit_sha: "abcdef1234567890".to_string(),
        timeout_seconds: 1800,
        steps: vec![
            StepDefinition {
                name: "Build App".to_string(),
                run: Some("cargo build --release".to_string()),
                container_build: None,
                when: Some("on_success".to_string()),
                env: None,
            },
            StepDefinition {
                name: "Build and Push Container".to_string(),
                run: None,
                container_build: Some(ContainerBuildConfig {
                    image: "ghcr.io/my-org/coffee-be".to_string(),
                    dockerfile: Some("Dockerfile".to_string()),
                    tags: vec!["latest".to_string(), "v1.0.0".to_string()],
                    registry_auth: None,
                }),
                when: Some("on_success".to_string()),
                env: None,
            },
        ],
        secrets,
        env,
    };

    let json_str = serde_json::to_string(&assign_msg).expect("Serialize ControllerToAgentMessage");
    let deserialized: ControllerToAgentMessage =
        serde_json::from_str(&json_str).expect("Deserialize ControllerToAgentMessage");

    match deserialized {
        ControllerToAgentMessage::AssignJob {
            job_id: parsed_job_id,
            steps,
            secrets: parsed_secrets,
            ..
        } => {
            assert_eq!(parsed_job_id, job_id);
            assert_eq!(steps.len(), 2);
            assert_eq!(parsed_secrets.get("API_KEY").unwrap(), "sec_live_998877");
        }
        _ => panic!("Expected AssignJob variant"),
    }

    // 2. Agent to Controller: LogStream & JobFinished
    let log_msg = AgentToControllerMessage::LogStream {
        job_id,
        step_index: 0,
        stream: "stdout".to_string(),
        line: "Compiling exaforge v0.1.0".to_string(),
    };
    let log_json = serde_json::to_string(&log_msg).expect("Serialize LogStream");
    assert!(log_json.contains("\"type\":\"LogStream\""));

    let mut outputs = HashMap::new();
    outputs.insert("IMAGE_DIGEST".to_string(), "sha256:11223344".to_string());

    let finished_msg = AgentToControllerMessage::JobFinished {
        job_id,
        status: "success".to_string(),
        outputs,
        step_summaries: "## Build Succeeded\n- 12 targets built".to_string(),
        error_message: None,
    };
    let finished_json = serde_json::to_string(&finished_msg).expect("Serialize JobFinished");
    let deserialized_finished: AgentToControllerMessage =
        serde_json::from_str(&finished_json).expect("Deserialize JobFinished");

    match deserialized_finished {
        AgentToControllerMessage::JobFinished { status, outputs, .. } => {
            assert_eq!(status, "success");
            assert_eq!(outputs.get("IMAGE_DIGEST").unwrap(), "sha256:11223344");
        }
        _ => panic!("Expected JobFinished variant"),
    }
}

#[test]
fn test_secret_masker_in_log_stream() {
    let mut secrets = HashMap::new();
    secrets.insert("DB_PASS".to_string(), "p@ssw0rd!123".to_string());
    secrets.insert("SECRET_TOKEN".to_string(), "ghp_VerySecretGitHubToken99".to_string());
    secrets.insert("SHORT".to_string(), "ab".to_string()); // Ignored if <= 3 chars

    let masker = SecretMasker::new(&secrets);

    let raw_log = "Connecting to postgres://admin:p@ssw0rd!123@db.prod with token ghp_VerySecretGitHubToken99 (ab)";
    let masked = masker.mask(raw_log);

    assert_eq!(
        masked,
        "Connecting to postgres://admin:***@db.prod with token *** (ab)"
    );
    assert!(!masked.contains("p@ssw0rd!123"));
    assert!(!masked.contains("ghp_VerySecretGitHubToken99"));
}

#[test]
fn test_monorepo_path_and_tag_filtering() {
    // 1. Monorepo Path Filtering using glob
    let fe_filter = glob::Pattern::new("apps/fe/**").unwrap();
    let be_filter = glob::Pattern::new("apps/be/**").unwrap();

    let touched_fe = "apps/fe/src/main.ts";
    let touched_be = "apps/be/Cargo.toml";
    let touched_shared = "packages/common/index.ts";

    assert!(fe_filter.matches(touched_fe));
    assert!(!fe_filter.matches(touched_be));
    assert!(!fe_filter.matches(touched_shared));

    assert!(be_filter.matches(touched_be));
    assert!(!be_filter.matches(touched_fe));

    // 2. Tag prefix pattern matching
    let tag_fe_filter = glob::Pattern::new("fe-v*").unwrap();
    let tag_be_filter = glob::Pattern::new("be-v*").unwrap();

    assert!(tag_fe_filter.matches("fe-v1.2.3"));
    assert!(!tag_fe_filter.matches("be-v1.2.3"));
    assert!(tag_be_filter.matches("be-v2.0.0-rc1"));
}

#[test]
fn test_pipeline_spec_concurrency_and_dependencies() {
    let pipeline_yaml = r#"
name: "Production Pipeline"
concurrency:
  group: "deploy-coffee-be"
  cancel_in_progress: true
inputs:
  app_name:
    default: "coffee-backend"
    description: "Application name"
  rollout_timeout:
    default: "300"
    description: "Timeout for kubernetes rollout"
jobs:
  lint:
    name: "Linting"
    tags: ["linux"]
    steps:
      - name: "Run Clippy"
        run: "cargo clippy"
  test:
    name: "Unit & Integration Tests"
    tags: ["linux", "docker"]
    needs: ["lint"]
    steps:
      - name: "Test Execution"
        run: "cargo test --all"
  build:
    name: "Container Image Build"
    tags: ["linux", "docker"]
    needs: ["test"]
    steps:
      - name: "Docker Build"
        container_build:
          image: "registry.company.com/${app_name}"
          tags: ["${app_name}-${rollout_timeout}"]
  deploy:
    name: "Kubernetes Deploy"
    tags: ["linux", "k8s"]
    needs: ["build"]
    steps:
      - name: "Kubectl Rollout"
        run: "kubectl rollout status deployment/${app_name} --timeout=${rollout_timeout}s"
"#;

    let config = parse_pipeline_config(pipeline_yaml).expect("Parse pipeline config");
    assert_eq!(config.name, "Production Pipeline");

    let conc = config.concurrency.expect("Concurrency config exists");
    assert_eq!(conc.group, "deploy-coffee-be");
    assert!(conc.cancel_in_progress);

    assert_eq!(config.jobs.len(), 4);
    assert_eq!(config.jobs.get("deploy").unwrap().tags, vec!["linux", "k8s"]);

    let sorted = topological_sort_jobs(&config.jobs).expect("Topological sort DAG");
    assert_eq!(sorted, vec!["lint", "test", "build", "deploy"]);

    // Test input variable interpolation
    let mut inputs = HashMap::new();
    inputs.insert("app_name".to_string(), "coffee-service".to_string());
    inputs.insert("rollout_timeout".to_string(), "600".to_string());

    let interpolated_cmd = interpolate_variables(
        "kubectl rollout status deployment/${app_name} --timeout=${rollout_timeout}s",
        &inputs,
    );
    assert_eq!(
        interpolated_cmd,
        "kubectl rollout status deployment/coffee-service --timeout=600s"
    );
}

#[test]
fn test_agent_token_generation_and_crypto() {
    let token = generate_agent_token();
    assert!(token.starts_with("agt_"));

    let master_key = [99u8; 32];
    let secret_val = "prod_db_super_secret_credentials_!@#$";

    let (enc, iv, tag) = encrypt_secret(secret_val, &master_key).expect("Encrypt secret");
    let dec = decrypt_secret(&enc, &iv, &tag, &master_key).expect("Decrypt secret");
    assert_eq!(dec, secret_val);

    let hmac_payload = b"{\"event\":\"push\"}";
    let webhook_secret = "git_hmac_secret_key";
    let is_valid = verify_hmac_sha256(hmac_payload, webhook_secret, "invalid_sig");
    assert!(!is_valid);
}

