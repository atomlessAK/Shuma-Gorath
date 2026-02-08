# 🐙 Observability & Grafana

## 🐙 Prometheus Metrics Endpoint

Shuma-Gorath exposes Prometheus-compatible metrics at:

```
GET /metrics
```

This endpoint is unauthenticated for Prometheus compatibility. Restrict access at the network edge if required.

### 🐙 Metrics Included

- `bot_trap_requests_total`
- `bot_trap_bans_total{reason="..."}`
- `bot_trap_blocks_total`
- `bot_trap_challenges_total`
- `bot_trap_challenge_served_total`
- `bot_trap_challenge_solved_total`
- `bot_trap_challenge_incorrect_total`
- `bot_trap_challenge_expired_replay_total`
- `bot_trap_whitelisted_total`
- `bot_trap_test_mode_actions_total`
- `bot_trap_maze_hits_total`
- `bot_trap_active_bans`
- `bot_trap_test_mode_enabled`

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
2. Build panels for requests total, bans by reason, active bans, challenges/blocks over time, and test mode status

## 🐙 Spin Cloud Monitoring

```bash
spin cloud logs
spin cloud apps info
spin cloud apps metrics
```
