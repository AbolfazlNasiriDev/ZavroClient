#!/usr/bin/env bash
# zavro-tun.sh — Zavro Client TUN helper.
# Invoked via pkexec as root:  pkexec /path/to/zavro-tun.sh up|down
# Creates / removes the tun0 interface used by the Xray TUN inbound
# (10.0.0.1/30). Exits non-zero with a message on stderr on any failure.
# NOTE: untested on the build VM (no pkexec, no /dev/net/tun there);
# verify on Fedora 45 before release.
set -euo pipefail

IFACE="tun0"
ADDR="10.0.0.1/30"

die() { echo "zavro-tun: $*" >&2; exit 1; }

command -v ip >/dev/null 2>&1 || die "ip command not found"

case "${1:-}" in
  up)
    if ip link show "$IFACE" >/dev/null 2>&1; then
      echo "zavro-tun: $IFACE already exists"
    else
      ip tuntap add dev "$IFACE" mode tun || die "failed to create $IFACE"
    fi
    ip addr flush dev "$IFACE" || true
    ip addr add "$ADDR" dev "$IFACE" || die "failed to assign $ADDR"
    ip link set dev "$IFACE" up mtu 1500 || die "failed to bring $IFACE up"
    # Route all traffic through the tunnel device; Xray's TUN inbound
    # handles the actual forwarding. Keep the default route untouched
    # so the proxy itself stays reachable.
    echo "zavro-tun: $IFACE up ($ADDR)"
    ;;
  down)
    if ip link show "$IFACE" >/dev/null 2>&1; then
      ip link set dev "$IFACE" down || true
      ip tuntap del dev "$IFACE" mode tun || ip link delete "$IFACE" || true
      echo "zavro-tun: $IFACE removed"
    else
      echo "zavro-tun: $IFACE does not exist, nothing to do"
    fi
    ;;
  *)
    die "usage: $0 up|down"
    ;;
esac
