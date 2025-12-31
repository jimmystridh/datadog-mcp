//! Infrastructure and Kubernetes tools

use crate::state::ToolContext;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use tracing::info;

pub async fn get_infrastructure(ctx: ToolContext) -> anyhow::Result<Value> {
    info!("Getting infrastructure information");

    let api = ctx.infrastructure_api();
    let result = api.list_hosts().await;

    tool_response_with_fields!(
        result,
        "infrastructure",
        ctx,
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

pub async fn get_tags(ctx: ToolContext, source: Option<String>) -> anyhow::Result<Value> {
    info!("Getting host tags");

    let api = ctx.infrastructure_api();
    let result = api.get_tags(source.as_deref()).await;

    tool_response_with_fields!(
        result,
        "tags",
        ctx,
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
) -> anyhow::Result<Value> {
    use std::time::{SystemTime, UNIX_EPOCH};

    info!(
        "Getting Kubernetes deployments{}",
        namespace
            .as_ref()
            .map(|ns| format!(" in namespace: {}", ns))
            .unwrap_or_default()
    );

    // Query for deployment replicas in the last 5 minutes
    let to_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let from_ts = to_ts - 300; // 5 minutes ago

    // Build query with optional namespace filter
    let namespace_filter = namespace
        .as_ref()
        .map(|ns| format!(",kube_namespace:{}", ns))
        .unwrap_or_default();

    let query = format!(
        "avg:kubernetes_state.deployment.replicas_desired{{*{}}} by {{kube_deployment,kube_namespace,kube_cluster_name}}",
        namespace_filter
    );

    // Use existing metrics API
    let api = ctx.metrics_api();
    let result = api.query_metrics(from_ts, to_ts, &query).await;

    tool_response_with_fields!(
        result,
        "kubernetes_deployments",
        ctx,
        data,
        {
            let mut unique_deployment_names = HashSet::new();
            let mut unique_namespaces = HashSet::new();
            if let Some(series) = &data.series {
                for s in series {
                    if let Some(scope) = &s.scope {
                        for tag in scope.split(',') {
                            if let Some((key, value)) = tag.split_once(':') {
                                if key == "kube_deployment" {
                                    unique_deployment_names.insert(value.to_string());
                                } else if key == "kube_namespace" {
                                    unique_namespaces.insert(value.to_string());
                                }
                            }
                        }
                    }
                }
            }
            format!(
                "Found {} deployments across {} namespaces",
                unique_deployment_names.len(),
                unique_namespaces.len()
            )
        },
        {
            // Extract deployment information from series
            let mut deployments = Vec::new();
            let mut unique_deployment_names = HashSet::new();
            let mut unique_namespaces = HashSet::new();

            if let Some(series) = &data.series {
                for s in series {
                    // Parse tags from scope
                    let mut tags = HashMap::new();
                    if let Some(scope) = &s.scope {
                        for tag in scope.split(',') {
                            if let Some((key, value)) = tag.split_once(':') {
                                tags.insert(key.to_string(), value.to_string());
                            }
                        }
                    }

                    let deployment = tags
                        .get("kube_deployment")
                        .cloned()
                        .unwrap_or_else(|| "unknown".to_string());
                    let ns = tags
                        .get("kube_namespace")
                        .cloned()
                        .unwrap_or_else(|| "unknown".to_string());
                    let cluster = tags
                        .get("kube_cluster_name")
                        .cloned()
                        .unwrap_or_else(|| "unknown".to_string());

                    unique_deployment_names.insert(deployment.clone());
                    unique_namespaces.insert(ns.clone());

                    deployments.push(json!({
                        "deployment": deployment,
                        "namespace": ns,
                        "cluster": cluster,
                    }));
                }
            }

            let mut deployment_names: Vec<String> = unique_deployment_names.into_iter().collect();
            deployment_names.sort();

            let mut namespace_list: Vec<String> = unique_namespaces.into_iter().collect();
            namespace_list.sort();

            json!({
                "deployments": deployments,
                "unique_deployment_names": deployment_names,
                "unique_namespaces": namespace_list,
            })
        }
    )
}
