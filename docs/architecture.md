# Architecture

grainx keeps local collection, remote collection, presentation, and export on the same snapshot model.

## Runtime paths

Local monitor:

SystemMonitor -> StatsSnapshot -> TUI

Local or remote export:

MetricBackend -> StatsSnapshot -> JSON and CSV

Remote monitor:

HTTP agent (/metrics) -> MetricBackend -> TUI

The HTTP agent is a separate process path that exposes one health endpoint and one JSON metrics endpoint. It is a transport for host metrics, not a general-purpose API or an AI agent.

## Module boundaries

| Area | Main modules | Responsibility |
| --- | --- | --- |
| CLI | src/cli.rs, src/main.rs | Parse subcommands and resolve defaults |
| Collection | src/monitor.rs, src/network.rs | Read host metrics, process data, and alerts |
| Remote service | src/agent.rs, src/metrics.rs | Serve or fetch a StatsSnapshot |
| Presentation | src/tui.rs, src/ui.rs, src/rendering.rs, src/theme.rs | Draw the terminal dashboard and controls |
| Data and analytics | src/export.rs, src/export_cmd.rs, src/analytics.rs, src/performance.rs | Serialize snapshots and calculate derived values |
| Configuration | src/config.rs | Load file values and apply environment/CLI overrides |

## Deliberate limitations

- The agent uses HTTP with no authentication, TLS, or rate limiting.
- The formula evaluator is a prototype with whitespace-separated, left-to-right arithmetic; it is not a full expression language.
- CI currently verifies Linux only. Cross-platform behavior still needs a release validation matrix.
- Process termination follows the permissions and platform behavior of the user running the process.