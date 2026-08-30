//! Infrastructure and Kubernetes tools

use crate::response::{CachePolicy, ToolOutput};
use crate::state::ToolContext;
use datadog_api::models::TimeseriesFormulaQueryResponse;
use datadog_api::TimestampSecs;
use serde::Serialize;
use serde_json::json;
use std::collections::{BTreeSet, HashMap};
use std::fmt;
use tracing::info;

#[derive(Debug, Clone, PartialEq, Eq)]
struct KubernetesNamespace(String);

impl TryFrom<String> for KubernetesNamespace {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let valid = !value.is_empty()
            && value.len() <= 63
            && value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            && value
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphanumeric)
            && value
                .as_bytes()
                .last()
                .is_some_and(u8::is_ascii_alphanumeric);
        if !valid {
            anyhow::bail!("namespace must be a valid lowercase DNS-1123 label");
        }
        Ok(Self(value))
    }
}

impl fmt::Display for KubernetesNamespace {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Serialize)]
struct DeploymentState {
    deployment: String,
    namespace: String,
    cluster: String,
    desired_replicas: Option<f64>,
}

#[derive(Debug, Serialize)]
struct DeploymentSnapshot {
    deployments: Vec<DeploymentState>,
    unique_deployment_names: Vec<String>,
    unique_namespaces: Vec<String>,
}

impl DeploymentSnapshot {
    fn from_response(response: &TimeseriesFormulaQueryResponse) -> Self {
        let mut deployments = Vec::new();
        let mut deployment_names = BTreeSet::new();
        let mut namespaces = BTreeSet::new();

        if let Some(data) = &response.data {
            for (index, series) in data.attributes.series.iter().enumerate() {
                let tags: HashMap<_, _> = series
                    .group_tags
                    .iter()
                    .filter_map(|tag| tag.split_once(':'))
                    .collect();
                let deployment = tags
                    .get("kube_deployment")
                    .copied()
                    .unwrap_or("unknown")
                    .to_string();
                let namespace = tags
                    .get("kube_namespace")
                    .copied()
                    .unwrap_or("unknown")
                    .to_string();
                let cluster = tags
                    .get("kube_cluster_name")
                    .copied()
                    .unwrap_or("unknown")
                    .to_string();
                let desired_replicas = data
                    .attributes
                    .values
                    .get(index)
                    .and_then(|values| values.iter().rev().flatten().next())
                    .copied();

                deployment_names.insert(deployment.clone());
                namespaces.insert(namespace.clone());
                deployments.push(DeploymentState {
                    deployment,
                    namespace,
                    cluster,
                    desired_replicas,
                });
            }
        }

        Self {
            deployments,
            unique_deployment_names: deployment_names.into_iter().collect(),
            unique_namespaces: namespaces.into_iter().collect(),
        }
    }

    fn summary(&self) -> String {
        format!(
            "Found {} deployments across {} namespaces",
            self.unique_deployment_names.len(),
            self.unique_namespaces.len()
        )
    }
}

pub async fn get_infrastructure(ctx: ToolContext) -> anyhow::Result<ToolOutput> {
    info!("Getting infrastructure information");

    let api = ctx.infrastructure_api();
    let result = api.list_hosts().await;

    tool_response_with_fields!(
        result,
        cache("infrastructure"),
        data,
        {
            let hosts = data.host_list.as_ref().map(|h| h.len()).unwrap_or(0);
            let active_hosts = data
                .host_list
                .as_ref()
                .map(|hosts| hosts.iter().filter(|h| h.up.unwrap_or(false)).count())
                .unwrap_or(0);
            format!("Found {} hosts ({} active)", hosts, active_hosts)
        },
        {
            let hosts = data.host_list.clone().unwrap_or_default();
            let active_hosts = hosts.iter().filter(|h| h.up.unwrap_or(false)).count();
            let total_hosts = hosts.len();
            json!({
                "total_hosts": total_hosts,
                "active_hosts": active_hosts,
                "inactive_hosts": total_hosts.saturating_sub(active_hosts),
            })
        }
    )
}

pub async fn get_tags(ctx: ToolContext, source: Option<String>) -> anyhow::Result<ToolOutput> {
    info!("Getting host tags");

    let api = ctx.infrastructure_api();
    let result = api.get_tags(source.as_deref()).await;

    tool_response_with_fields!(
        result,
        cache("tags"),
        data,
        {
            let tags = data.tags.as_ref().map(|t| t.len()).unwrap_or(0);
            format!("Retrieved tags for {} hosts", tags)
        },
        {
            let tags = data.tags.as_ref().map(|t| t.len()).unwrap_or(0);
            json!({
                "host_count": tags,
                "source": source.unwrap_or_else(|| "all".to_string()),
            })
        }
    )
}

pub async fn get_kubernetes_deployments(
    ctx: ToolContext,
    namespace: Option<String>,
) -> anyhow::Result<ToolOutput> {
    let namespace = namespace.map(KubernetesNamespace::try_from).transpose()?;

    info!(
        "Getting Kubernetes deployments{}",
        namespace
            .as_ref()
            .map(|ns| format!(" in namespace: {}", ns))
            .unwrap_or_default()
    );

    let to_ts = TimestampSecs::now().as_secs();
    let from_ts = to_ts - 300;

    // Build query with optional namespace filter
    let namespace_filter = namespace
        .as_ref()
        .map(|namespace| format!("kube_namespace:{namespace}"))
        .unwrap_or_else(|| "*".to_string());

    let query = format!(
        "avg:kubernetes_state.deployment.replicas_desired{{{}}} by {{kube_deployment,kube_namespace,kube_cluster_name}}",
        namespace_filter
    );

    // Use existing metrics API
    let api = ctx.metrics_api();
    let data = api.query_metrics(from_ts, to_ts, &query).await?;
    let snapshot = DeploymentSnapshot::from_response(&data);
    ToolOutput::from_data(
        &data,
        snapshot.summary(),
        serde_json::to_value(snapshot)?,
        CachePolicy::Store("kubernetes_deployments"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kubernetes_namespaces_are_dns_labels() {
        assert!(KubernetesNamespace::try_from("production".to_string()).is_ok());
        assert!(KubernetesNamespace::try_from("Production".to_string()).is_err());
        assert!(KubernetesNamespace::try_from("-invalid".to_string()).is_err());
    }
}
