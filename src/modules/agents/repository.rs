use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::pkg::errors::AppError;
use super::dto::AgentRecord;

#[async_trait]
pub trait AgentRepository: Send + Sync {
    async fn create_agent(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        name: &str,
        agent_token: &str,
        tags: serde_json::Value,
    ) -> Result<AgentRecord, AppError>;

    async fn find_by_token(&self, pool: &PgPool, token: &str) -> Result<Option<AgentRecord>, AppError>;
    async fn find_by_id(&self, pool: &PgPool, id: Uuid) -> Result<Option<AgentRecord>, AppError>;
    async fn list_agents(&self, pool: &PgPool) -> Result<Vec<AgentRecord>, AppError>;

    async fn update_status(
        &self,
        pool: &PgPool,
        id: Uuid,
        status: &str,
        socket_id: Option<&str>,
        os_info: Option<serde_json::Value>,
    ) -> Result<(), AppError>;

    async fn update_heartbeat(&self, pool: &PgPool, id: Uuid) -> Result<(), AppError>;

    async fn find_matching_online_agent(
        &self,
        pool: &PgPool,
        required_tags: &[String],
    ) -> Result<Option<AgentRecord>, AppError>;
}

pub struct PostgresAgentRepository;

#[async_trait]
impl AgentRepository for PostgresAgentRepository {
    async fn create_agent(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        name: &str,
        agent_token: &str,
        tags: serde_json::Value,
    ) -> Result<AgentRecord, AppError> {
        let record = sqlx::query_as::<_, AgentRecord>(
            r#"
            INSERT INTO agents (name, agent_token, tags, status)
            VALUES ($1, $2, $3, 'offline')
            RETURNING id, name, agent_token, tags, status, socket_id, os_info, last_heartbeat_at, created_at, updated_at
            "#
        )
        .bind(name)
        .bind(agent_token)
        .bind(tags)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::database(format!("Failed to create agent: {}", e)))?;

        Ok(record)
    }

    async fn find_by_token(&self, pool: &PgPool, token: &str) -> Result<Option<AgentRecord>, AppError> {
        let record = sqlx::query_as::<_, AgentRecord>(
            "SELECT * FROM agents WHERE agent_token = $1"
        )
        .bind(token)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find agent by token: {}", e)))?;

        Ok(record)
    }

    async fn find_by_id(&self, pool: &PgPool, id: Uuid) -> Result<Option<AgentRecord>, AppError> {
        let record = sqlx::query_as::<_, AgentRecord>(
            "SELECT * FROM agents WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find agent: {}", e)))?;

        Ok(record)
    }

    async fn list_agents(&self, pool: &PgPool) -> Result<Vec<AgentRecord>, AppError> {
        let records = sqlx::query_as::<_, AgentRecord>(
            "SELECT * FROM agents ORDER BY status ASC, name ASC"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to list agents: {}", e)))?;

        Ok(records)
    }

    async fn update_status(
        &self,
        pool: &PgPool,
        id: Uuid,
        status: &str,
        socket_id: Option<&str>,
        os_info: Option<serde_json::Value>,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE agents
            SET status = $1,
                socket_id = $2,
                os_info = COALESCE($3, os_info),
                last_heartbeat_at = NOW(),
                updated_at = NOW()
            WHERE id = $4
            "#
        )
        .bind(status)
        .bind(socket_id)
        .bind(os_info)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to update agent status: {}", e)))?;

        Ok(())
    }

    async fn update_heartbeat(&self, pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        sqlx::query("UPDATE agents SET last_heartbeat_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| AppError::database(format!("Failed to update heartbeat: {}", e)))?;

        Ok(())
    }

    async fn find_matching_online_agent(
        &self,
        pool: &PgPool,
        required_tags: &[String],
    ) -> Result<Option<AgentRecord>, AppError> {
        let agents = sqlx::query_as::<_, AgentRecord>(
            "SELECT * FROM agents WHERE status = 'online' ORDER BY last_heartbeat_at DESC"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("Failed to find online agents: {}", e)))?;

        // Filter agent whose tags contain all required_tags
        for agent in agents {
            if let Some(agent_tags_arr) = agent.tags.as_array() {
                let agent_tags: Vec<String> = agent_tags_arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();

                let matches_all = required_tags.iter().all(|req_tag| agent_tags.contains(req_tag));
                if matches_all {
                    return Ok(Some(agent));
                }
            }
        }

        Ok(None)
    }
}

