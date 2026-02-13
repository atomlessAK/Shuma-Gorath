# 🐙 Observability & Grafana

## 🐙 Prometheus Metrics Endpoint

Shuma-Gorath exposes Prometheus-compatible metrics at:

```
GET /metrics
```

This endpoint is unauthenticated for Prometheus compatibility. Restrict access at the network edge if required.

### 🐙 Metrics Included

- `bot_defence_requests_total`
- `bot_defence_bans_total{reason="..."}`
- `bot_defence_blocks_total`
- `bot_defence_challenges_total`
- `bot_defence_challenge_served_total`
- `bot_defence_challenge_solved_total`
- `bot_defence_challenge_incorrect_total`
- `bot_defence_challenge_expired_replay_total`
- `bot_defence_whitelisted_total`
- `bot_defence_test_mode_actions_total`
- `bot_defence_maze_hits_total`
- `bot_defence_active_bans`
- `bot_defence_test_mode_enabled`
- `bot_defence_botness_signal_state_total{signal="...",state="active|disabled|unavailable"}`
- `bot_defence_defence_mode_effective_total{module="rate|geo|js",configured="off|signal|enforce|both",signal_enabled="true|false",action_enabled="true|false"}`
- `bot_defence_edge_integration_mode_total{mode="off|advisory|authoritative"}`
- `bot_defence_provider_implementation_effective_total{capability="...",backend="internal|external",implementation="..."}`

## 🐙 Prometheus Scrape Example

```yaml
scrape_configs:
  - job_name: shuma-gorath
    static_configs:
      - targets: ["your-domain.example.com"]
    metrics_path: /metrics
```

## 🐙 Grafana Integration

1. Add Prometheus as a data source
2. Build panels for requests total, bans by reason, active bans, challenges/blocks over time, test mode status, and composability visibility (signal-state and effective-mode counters)

## 🐙 Botness Visibility

- Botness-driven challenge/maze events include:
  - active signal summary (`signals=...`)
  - full state summary (`signal_states=key:state:contribution,...`)
  - runtime metadata summary (`modes=rate=... geo=... js=... edge=...`)
  - provider summary (`providers=rate_limiter=... ban_store=... challenge_engine=... maze_tarpit=... fingerprint_signal=...`)
- Use this event context with the two composability metrics to distinguish:
  - intentional disabled signals (`state=disabled`),
  - unavailable inputs (`state=unavailable`), and
  - active contributors (`state=active`).

## 🐙 External Provider Cutover Monitoring

Use this when moving from internal-only to advisory/authoritative edge integration.

### 1. Mode and Provider Selection Checks

- Confirm configured edge mode is being observed:
  - `bot_defence_edge_integration_mode_total{mode="off|advisory|authoritative"}`
- Confirm active provider implementation by capability:
  - `bot_defence_provider_implementation_effective_total{capability="...",backend="...",implementation="..."}`
  - For `rate_limiter` external mode, expect `implementation="external_redis_with_internal_fallback"`.
  - For `ban_store` external mode, expect `implementation="external_redis_with_internal_fallback"`.
- Use `increase(...)` windows in PromQL to verify recent behavior rather than cumulative lifetime totals.

Example PromQL (last 10 minutes):

```promql
sum by (mode) (increase(bot_defence_edge_integration_mode_total[10m]))
```

```promql
sum by (capability, backend, implementation) (
  increase(bot_defence_provider_implementation_effective_total[10m])
)
```

### 2. Signal Health Checks

- Watch unavailable signal-state growth during external cutover:
  - `bot_defence_botness_signal_state_total{state="unavailable"}`
- For fingerprint migrations specifically, an unexpected rise in unavailable state after enablement is a rollback trigger.

Example PromQL (last 10 minutes):

```promql
sum by (signal, state) (increase(bot_defence_botness_signal_state_total[10m]))
```

### 3. Outcome Sanity Checks

Correlate provider/mode transitions with:

- `bot_defence_challenges_total`
- `bot_defence_blocks_total`
- admin event outcomes that include:
  - `signal_states=...`
  - `modes=... edge=...`
  - `providers=...`

If challenge/block behavior changes sharply without matching traffic or threat context, roll back to internal/`off` and investigate.

### 4. Minimum Alerting Guidance

During any external provider rollout, alert on:

- sustained increase in unavailable signal state,
- unexpected provider implementation label changes,
- sudden challenge/block jumps versus baseline.

## 🐙 Spin Cloud Monitoring

```bash
spin cloud logs
spin cloud apps info
spin cloud apps metrics
```
