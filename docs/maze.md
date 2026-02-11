# 🐙 Maze

The maze is a crawler trap that serves a recursive tree of synthetic pages. It is designed to capture bots that follow links aggressively or ignore robots policies.

## 🐙 How It Works

- Requests to maze paths return pages that link to more maze pages
- Each maze hit increments a counter
- When a crawler crosses the configured threshold, it can be auto-banned
- `robots.txt` now advertises active trap routes (`/maze/`, `/trap/`) and configured honeypot paths (for example `/instaban`)

## 🐙 Configuration

These fields are part of the runtime config (`/admin/config`):

- `maze_enabled` (bool) - Enable or disable the maze
- `maze_auto_ban` (bool) - Auto-ban after threshold
- `maze_auto_ban_threshold` (u32) - Number of maze pages before auto-ban

## 🐙 Admin Endpoint

- `GET /admin/maze` - Returns maze stats for the dashboard

## 🐙 Metrics

- `bot_defence_maze_hits_total` tracks total maze page hits

## 🐙 Notes

- If you do not want crawler trapping, set `maze_enabled` to `false`
- Auto-ban uses the `maze_crawler` reason in metrics and events
