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
| Metrics | Read-only | Read-only | v2 timeseries query plus active list/metadata; no metric submission or metadata updates. |
| Database Monitoring | Query metrics + sample search | Read-only | Scalar metrics ranked by query signature; activity/session and explain-plan samples from the databasequery index. API keys, no browser; sample API requires an unscoped app key. |
| Monitors | CRUD + search + legacy group resolve | CRUD + search + legacy group resolve | Group resolution uses Datadog's undocumented, unversioned endpoint and is explicitly marked experimental. |
| Slack integration | Channel CRUD | Channel CRUD + monitor notification audit | Workspace installation remains an OAuth UI operation; tools manage channels within an existing account and audit monitor handles. |
| Dashboards | CRUD | CRUD |  |
| Logs | Search-only | Search-only | No indexes/pipelines/archives. |
| Events | Read + create (list/get/post) | Read + create (list/get/post) |  |
| Infrastructure (hosts/tags) | Read-only | Read-only | Host list + tags only. |
| Kubernetes deployments | No | Derived query tool | MCP tool uses a metrics query over k8s state. |
| Kubernetes workload scaling | No public API | Read-only UI endpoint | Uses a local browser session through `cookie-scoop`; the undocumented endpoint has no compatibility guarantee. |
| Downtimes | Partial (list/create/cancel) | Partial (list/create/cancel) | No update. |
| Synthetics | Partial (list/get/locations/create/update/trigger/delete) | Partial | No results/history. |
| Security Monitoring | Rules list-only | Rules list-only | No signals/findings. |
| Incidents | List-only | List-only | Single bounded page with next offset returned. |
| SLOs | List-only | List-only | No create/update/history. |
| Notebooks | List-only | List-only |  |
| Teams | List-only | List-only | Page number and size supported. |
| Users | List-only | List-only | Current v2 endpoint with page number and size. |
| APM/Spans | Search + service/dependency reads | Not exposed | Crate uses `/api/v2/spans/events/search`, `/api/v2/apm/services`, and `/api/v1/service_dependencies`; trace intake belongs on an Agent/OTLP pipeline. |

## MCP-Only Utilities

- `validate_api_key`
- `analyze_data` (cache analysis)
- `cleanup_cache`

## Major Gaps (not in crate or MCP)

- RUM (events, analytics, session replay)
- CI/CD & Testing (CI visibility, test runs, pipelines)
- Cloud integrations (AWS/Azure/GCP, cost management)
- Advanced monitoring (network, processes, containers; DBM analytics and configuration beyond query metrics/sample search)
- Security (signals, findings, compliance, audit logs)
- Admin/Org (orgs, roles/permissions, API/app keys, usage/billing)
- Logs management (pipelines, processors, indexes, archives)
- Automation (webhooks, non-Slack integration configs, custom checks)
- Other (service catalog, SLA, workflows, case management, status pages)
