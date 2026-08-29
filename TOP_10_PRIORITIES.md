# Top 10 Datadog API Priorities

This list reflects the current implementation. Trace intake is intentionally excluded: applications should send telemetry through a Datadog Agent, OpenTelemetry Collector, or supported intake library rather than an MCP control-plane client.

1. **Expose indexed span search and APM services through MCP**
   - The reusable crate now supports current span search, APM service list, and service-dependency endpoints.
   - Add bounded MCP tools with explicit cursors and environment filters.

2. **Service Catalog read support**
   - List and inspect service definitions, ownership, lifecycle, and links.
   - Keep writes behind the existing `--allow-write` control.

3. **SLO history and error budgets**
   - Add SLO detail, history, corrections, and error-budget queries before CRUD.
   - These reads provide higher operational value with lower mutation risk.

4. **Security signals and findings**
   - Add cursor-paginated signal and cloud-security finding reads.
   - Treat payloads as sensitive and never persist them in the response cache.

5. **Usage and cost visibility**
   - Add estimated cost, hourly usage, and product-level usage APIs.
   - Return bounded periods and explicit pagination to avoid large MCP payloads.

6. **Log management reads**
   - Inspect pipelines, processors, indexes, and archives.
   - Add write operations only with lossless models and focused guardrails.

7. **Synthetics results and history**
   - Add API, browser, and mobile test result summaries and detailed result reads.
   - Preserve test-type-specific fields as the existing update path now does.

8. **Cloud and integration inventory**
   - List AWS, Azure, GCP, and integration configuration without exposing secrets.
   - Avoid key-management tools in the MCP surface unless a separate privileged mode is designed.

9. **Process, container, and network inventory**
   - Add bounded infrastructure queries that complement hosts and Kubernetes deployment metrics.

10. **Metric metadata and tag-configuration management**
    - Add v2 tag configuration, volumes, cardinality, and metadata updates.
    - Keep metric submission out of the control-plane MCP server unless a concrete operational use case requires it.

## Implementation Rules

- Prefer current Datadog v2 endpoints where Datadog recommends them.
- Use the official OpenAPI-generated client or verify every custom endpoint and model against the current specification.
- Every list/search tool must expose pagination and return the next cursor or offset.
- Reads may retry transient failures; writes must not retry automatically unless the operation has a documented idempotency mechanism.
- New write tools must be annotated accurately and remain disabled without `--allow-write`.
- Sensitive responses must remain inline and must not enter the local response cache.
