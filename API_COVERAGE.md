# Datadog API Coverage Matrix

This file tracks current coverage based on the code in `datadog-api/src/apis` and
`datadog-mcp/src/tools`. It distinguishes between the reusable Rust client crate
and the MCP tool surface.

## Coverage Matrix (crate vs MCP)

Legend:
- CRUD = create/read/update/delete
- Read-only = list/query/get only
- Partial = some write ops, but not full CRUD
- Not exposed = not available via MCP tools

| Area | datadog-api crate | MCP tools | Notes |
| --- | --- | --- | --- |
| Metrics | Read-only | Read-only | Query/list/metadata only; no metric submission or metadata updates. |
| Monitors | CRUD + search | CRUD + search |  |
| Dashboards | CRUD | CRUD |  |
| Logs | Search-only | Search-only | No indexes/pipelines/archives. |
| Events | Read + create (list/get/post) | Read + create (list/get/post) |  |
| Infrastructure (hosts/tags) | Read-only | Read-only | Host list + tags only. |
| Kubernetes deployments | No | Derived query tool | MCP tool uses a metrics query over k8s state. |
| Downtimes | Partial (list/create/cancel) | Partial (list/create/cancel) | No update. |
| Synthetics | Partial (list/get/locations/create/update/trigger/delete) | Partial | No results/history. |
| Security Monitoring | Rules list-only | Rules list-only | No signals/findings. |
| Incidents | List-only | List-only | MCP wraps list-all pagination. |
| SLOs | List-only | List-only | No create/update/history. |
| Notebooks | List-only | List-only |  |
| Teams | List-only | List-only |  |
| Users | List-only | List-only |  |
| APM/Traces | Read/write (submit/search/get/stats/dependencies/services) | Not exposed | Crate only. |

## MCP-Only Utilities

- `validate_api_key`
- `analyze_data` (cache analysis)
- `cleanup_cache`

## Major Gaps (not in crate or MCP)

- RUM (events, analytics, session replay)
- CI/CD & Testing (CI visibility, test runs, pipelines)
- Cloud integrations (AWS/Azure/GCP, cost management)
- Advanced monitoring (network, database, processes, containers)
- Security (signals, findings, compliance, audit logs)
- Admin/Org (orgs, roles/permissions, API/app keys, usage/billing)
- Logs management (pipelines, processors, indexes, archives)
- Automation (webhooks, integration configs, custom checks)
- Other (service catalog, SLA, workflows, case management, status pages)
