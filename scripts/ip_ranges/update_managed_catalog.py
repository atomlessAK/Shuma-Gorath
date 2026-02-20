#!/usr/bin/env python3
"""Refresh config/managed_ip_ranges.json from official machine-readable sources.

Guardrails:
- HTTPS + source-host allowlist only.
- Strict schema parsing for each source.
- CIDR parsing with broad-prefix rejection.
- Per-set entry caps and growth-delta guard.
"""

from __future__ import annotations

import argparse
import hashlib
import ipaddress
import json
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Tuple
from urllib.parse import urlparse
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_OUTPUT_PATH = ROOT / "config" / "managed_ip_ranges.json"

ALLOWED_SOURCE_HOSTS = {"openai.com", "api.github.com"}
MIN_IPV4_PREFIX_LEN = 8
MIN_IPV6_PREFIX_LEN = 24
MAX_CIDRS_PER_SET = 4096
MAX_SET_GROWTH_FACTOR = 4.0
MAX_SET_GROWTH_ABS_DELTA = 2048
USER_AGENT = "shuma-ip-range-catalog-updater/1.0"


@dataclass(frozen=True)
class SourceSpec:
    set_id: str
    label: str
    provider: str
    source_url: str
    parser: str
    payload_key: Optional[str] = None


SOURCES: Tuple[SourceSpec, ...] = (
    SourceSpec(
        set_id="openai_gptbot",
        label="OpenAI GPTBot",
        provider="openai",
        source_url="https://openai.com/gptbot.json",
        parser="openai_prefixes",
    ),
    SourceSpec(
        set_id="openai_oai_searchbot",
        label="OpenAI OAI-SearchBot",
        provider="openai",
        source_url="https://openai.com/searchbot.json",
        parser="openai_prefixes",
    ),
    SourceSpec(
        set_id="openai_chatgpt_user",
        label="OpenAI ChatGPT-User",
        provider="openai",
        source_url="https://openai.com/chatgpt-user.json",
        parser="openai_prefixes",
    ),
    SourceSpec(
        set_id="github_copilot",
        label="GitHub Copilot",
        provider="github",
        source_url="https://api.github.com/meta",
        parser="json_array_key",
        payload_key="copilot",
    ),
)


def err(message: str) -> None:
    print(f"ERROR: {message}", file=sys.stderr)


def normalize_timestamp(raw: Optional[str]) -> Tuple[Optional[str], Optional[int]]:
    if raw is None:
        return None, None
    value = raw.strip()
    if not value:
        return None, None
    if value.endswith("Z"):
        value = value[:-1] + "+00:00"
    try:
        parsed = datetime.fromisoformat(value)
    except ValueError:
        return None, None
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    parsed = parsed.astimezone(timezone.utc)
    return parsed.strftime("%Y-%m-%dT%H:%M:%SZ"), int(parsed.timestamp())


def ensure_source_url_allowed(url: str) -> None:
    parsed = urlparse(url)
    if parsed.scheme.lower() != "https":
        raise ValueError(f"source must use https: {url}")
    host = (parsed.hostname or "").lower()
    if host not in ALLOWED_SOURCE_HOSTS:
        raise ValueError(f"source host is not allowlisted: {host or '(empty)'}")


def fetch_source_json(url: str) -> Tuple[Dict[str, Any], Dict[str, str]]:
    ensure_source_url_allowed(url)
    request = Request(
        url,
        headers={
            "User-Agent": USER_AGENT,
            "Accept": "application/json",
        },
    )
    with urlopen(request, timeout=20) as response:  # nosec B310 (allowlisted https hosts only)
        payload = response.read().decode("utf-8")
        headers = {k.lower(): v for k, v in response.headers.items()}
    try:
        parsed = json.loads(payload)
    except json.JSONDecodeError as exc:
        raise ValueError(f"invalid JSON from {url}: {exc}") from exc
    if not isinstance(parsed, dict):
        raise ValueError(f"expected top-level object from {url}")
    return parsed, headers


def parse_openai_prefixes(payload: Dict[str, Any], source_url: str) -> Tuple[List[str], Optional[str], Optional[int]]:
    prefixes = payload.get("prefixes")
    if not isinstance(prefixes, list):
        raise ValueError(f"{source_url} missing 'prefixes' array")
    extracted: List[str] = []
    for index, entry in enumerate(prefixes):
        if not isinstance(entry, dict):
            raise ValueError(f"{source_url} prefixes[{index}] must be an object")
        ipv4 = entry.get("ipv4Prefix")
        ipv6 = entry.get("ipv6Prefix")
        if isinstance(ipv4, str):
            extracted.append(ipv4)
        elif isinstance(ipv6, str):
            extracted.append(ipv6)
        else:
            raise ValueError(
                f"{source_url} prefixes[{index}] must include 'ipv4Prefix' or 'ipv6Prefix'"
            )
    source_ts, source_ts_unix = normalize_timestamp(
        payload.get("creationTime") if isinstance(payload.get("creationTime"), str) else None
    )
    return extracted, source_ts, source_ts_unix


def parse_json_array_key(
    payload: Dict[str, Any],
    source_url: str,
    payload_key: Optional[str],
) -> Tuple[List[str], Optional[str], Optional[int]]:
    if not payload_key:
        raise ValueError("payload_key is required for parser=json_array_key")
    values = payload.get(payload_key)
    if not isinstance(values, list):
        raise ValueError(f"{source_url} missing '{payload_key}' array")
    extracted: List[str] = []
    for index, value in enumerate(values):
        if not isinstance(value, str):
            raise ValueError(f"{source_url} {payload_key}[{index}] must be a string")
        extracted.append(value)
    return extracted, None, None


def parse_cidr(value: str) -> ipaddress._BaseNetwork:
    cidr = value.strip()
    if not cidr:
        raise ValueError("CIDR entry is empty")
    try:
        network = ipaddress.ip_network(cidr, strict=True)
    except ValueError as exc:
        raise ValueError(f"invalid CIDR '{cidr}'") from exc
    if network.version == 4 and network.prefixlen < MIN_IPV4_PREFIX_LEN:
        raise ValueError(f"CIDR '{cidr}' too broad (min /{MIN_IPV4_PREFIX_LEN} for IPv4)")
    if network.version == 6 and network.prefixlen < MIN_IPV6_PREFIX_LEN:
        raise ValueError(f"CIDR '{cidr}' too broad (min /{MIN_IPV6_PREFIX_LEN} for IPv6)")
    return network


def canonicalize_cidrs(raw_cidrs: Iterable[str], set_id: str) -> List[str]:
    unique: Dict[str, ipaddress._BaseNetwork] = {}
    for raw in raw_cidrs:
        network = parse_cidr(raw)
        unique[str(network)] = network
    ordered = sorted(
        unique.values(),
        key=lambda net: (net.version, int(net.network_address), net.prefixlen),
    )
    if not ordered:
        raise ValueError(f"{set_id} produced an empty CIDR list")
    if len(ordered) > MAX_CIDRS_PER_SET:
        raise ValueError(
            f"{set_id} produced {len(ordered)} CIDRs (max {MAX_CIDRS_PER_SET})"
        )
    return [str(net) for net in ordered]


def version_for_set(
    now_date: str,
    cidrs: List[str],
    source_timestamp: Optional[str],
) -> str:
    hasher = hashlib.sha256()
    hasher.update("\n".join(cidrs).encode("utf-8"))
    hasher.update(b"\n")
    if source_timestamp:
        hasher.update(source_timestamp.encode("utf-8"))
    return f"{now_date}-{hasher.hexdigest()[:16]}"


def load_existing_counts(path: Path) -> Dict[str, int]:
    if not path.exists():
        return {}
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return {}
    if not isinstance(data, dict):
        return {}
    sets = data.get("sets")
    if not isinstance(sets, list):
        return {}
    counts: Dict[str, int] = {}
    for item in sets:
        if not isinstance(item, dict):
            continue
        set_id = item.get("id")
        cidrs = item.get("cidrs")
        if isinstance(set_id, str) and isinstance(cidrs, list):
            counts[set_id] = len(cidrs)
    return counts


def enforce_growth_guard(
    set_id: str,
    baseline_count: Optional[int],
    new_count: int,
    allow_large_delta: bool,
) -> None:
    if baseline_count is None or baseline_count <= 0:
        return
    if new_count <= baseline_count:
        return
    growth_ok = new_count <= int(baseline_count * MAX_SET_GROWTH_FACTOR) + 64
    delta_ok = (new_count - baseline_count) <= MAX_SET_GROWTH_ABS_DELTA
    if growth_ok and delta_ok:
        return
    if allow_large_delta:
        return
    raise ValueError(
        f"{set_id} growth guard tripped: baseline={baseline_count}, new={new_count}, "
        "rerun with --allow-large-delta after manual source verification"
    )


def build_catalog(existing_counts: Dict[str, int], allow_large_delta: bool) -> Dict[str, Any]:
    now = datetime.now(timezone.utc)
    now_iso = now.strftime("%Y-%m-%dT%H:%M:%SZ")
    now_unix = int(now.timestamp())
    today = now.strftime("%Y-%m-%d")

    sets_out: List[Dict[str, Any]] = []

    for source in SOURCES:
        payload, _headers = fetch_source_json(source.source_url)
        if source.parser == "openai_prefixes":
            raw_cidrs, source_ts, source_ts_unix = parse_openai_prefixes(payload, source.source_url)
        elif source.parser == "json_array_key":
            raw_cidrs, source_ts, source_ts_unix = parse_json_array_key(
                payload, source.source_url, source.payload_key
            )
        else:
            raise ValueError(f"unsupported parser: {source.parser}")

        cidrs = canonicalize_cidrs(raw_cidrs, source.set_id)
        enforce_growth_guard(
            source.set_id,
            existing_counts.get(source.set_id),
            len(cidrs),
            allow_large_delta=allow_large_delta,
        )

        sets_out.append(
            {
                "id": source.set_id,
                "label": source.label,
                "provider": source.provider,
                "source_url": source.source_url,
                "source_timestamp": source_ts,
                "source_timestamp_unix": source_ts_unix,
                "cidrs": cidrs,
                "version": version_for_set(today, cidrs, source_ts),
            }
        )

    return {
        "catalog_version": today,
        "generated_at": now_iso,
        "generated_at_unix": now_unix,
        "sets": sets_out,
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Refresh config/managed_ip_ranges.json from official OpenAI/GitHub "
            "machine-readable CIDR sources with strict validation guardrails."
        )
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=DEFAULT_OUTPUT_PATH,
        help=f"Output catalog path (default: {DEFAULT_OUTPUT_PATH})",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Validate sources and fail if generated output differs from current file.",
    )
    parser.add_argument(
        "--allow-large-delta",
        action="store_true",
        help="Allow large set-size growth beyond default anti-poisoning thresholds.",
    )
    return parser.parse_args()


def normalize_catalog_for_check(value: Dict[str, Any]) -> Dict[str, Any]:
    sets = value.get("sets")
    normalized_sets: List[Dict[str, Any]] = []
    if isinstance(sets, list):
        for item in sets:
            if not isinstance(item, dict):
                continue
            normalized_sets.append(
                {
                    "id": item.get("id"),
                    "label": item.get("label"),
                    "provider": item.get("provider"),
                    "source_url": item.get("source_url"),
                    "source_timestamp": item.get("source_timestamp"),
                    "source_timestamp_unix": item.get("source_timestamp_unix"),
                    "cidrs": item.get("cidrs"),
                    "version": item.get("version"),
                }
            )
    normalized_sets.sort(key=lambda entry: str(entry.get("id", "")))
    return {
        "catalog_version": value.get("catalog_version"),
        "sets": normalized_sets,
    }


def main() -> int:
    args = parse_args()
    output_path = args.output.resolve()
    existing_counts = load_existing_counts(output_path)
    try:
        catalog = build_catalog(
            existing_counts=existing_counts,
            allow_large_delta=args.allow_large_delta,
        )
    except Exception as exc:  # pylint: disable=broad-except
        err(str(exc))
        return 1

    rendered = json.dumps(catalog, indent=2) + "\n"

    if args.check:
        if not output_path.exists():
            err(f"--check failed: {output_path} does not exist")
            return 1
        try:
            current = json.loads(output_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            err(f"--check failed: {output_path} is invalid JSON ({exc})")
            return 1
        if normalize_catalog_for_check(current) != normalize_catalog_for_check(catalog):
            err(
                f"--check failed: {output_path} is stale; run this script to refresh managed ranges"
            )
            return 1
        print(f"OK: {output_path} is up to date")
        return 0

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(rendered, encoding="utf-8")
    print(f"Wrote managed IP range catalog: {output_path}")
    for set_obj in catalog["sets"]:
        print(
            f"- {set_obj['id']}: {len(set_obj['cidrs'])} CIDRs "
            f"(version={set_obj['version']})"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
