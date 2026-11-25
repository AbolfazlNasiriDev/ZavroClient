import { useMemo } from "react";
import type { TrafficPoint } from "../types";
import { formatBps } from "../api";

interface Props {
  points: TrafficPoint[];
  dlBps: number;
  ulBps: number;
}

const W = 720;
const H = 110;
const PAD = 6;

function buildPath(values: number[], max: number): string {
  if (values.length === 0) return "";
  const step = (W - PAD * 2) / Math.max(values.length - 1, 1);
  return values
    .map((v, i) => {
      const x = PAD + i * step;
      const y = H - PAD - (v / max) * (H - PAD * 2);
      return `${i === 0 ? "M" : "L"}${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
}

export default function TrafficChart({ points, dlBps, ulBps }: Props) {
  const { dlPath, ulPath, max } = useMemo(() => {
    const dl = points.map((p) => p.dl);
    const ul = points.map((p) => p.ul);
    const peak = Math.max(1, ...dl, ...ul);
    return {
      dlPath: buildPath(dl, peak),
      ulPath: buildPath(ul, peak),
      max: peak,
    };
  }, [points]);

  const gridYs = [0.25, 0.5, 0.75].map((f) => PAD + f * (H - PAD * 2));

  return (
    <div className="rounded-xl border border-adwaita-border bg-adwaita-panel px-3 pb-2 pt-2">
      <div className="flex items-center justify-between px-1 pb-1 text-xs text-adwaita-muted">
        <span>
          DL: {formatBps(dlBps)} | UL: {formatBps(ulBps)}
        </span>
        <span className="flex items-center gap-3">
          <span className="flex items-center gap-1">
            <span className="inline-block h-0.5 w-4 bg-adwaita-accent" /> Download
          </span>
          <span className="flex items-center gap-1">
            <span className="inline-block h-0.5 w-4 bg-adwaita-green" /> Upload
          </span>
          <span>peak {formatBps(max)}</span>
        </span>
      </div>
      {points.length < 2 ? (
        <div className="flex h-[110px] items-center justify-center text-xs text-adwaita-muted">
          Waiting for traffic data…
        </div>
      ) : (
        <svg
          viewBox={`0 0 ${W} ${H}`}
          preserveAspectRatio="none"
          className="h-[110px] w-full"
          role="img"
          aria-label="Live download and upload bandwidth chart"
        >
          {gridYs.map((y) => (
            <line
              key={y}
              x1={PAD}
              x2={W - PAD}
              y1={y}
              y2={y}
              stroke="#4a4a4a"
              strokeWidth={0.5}
              strokeDasharray="3 4"
            />
          ))}
          <path d={ulPath} fill="none" stroke="#33d17a" strokeWidth={1.5} />
          <path d={dlPath} fill="none" stroke="#3584e4" strokeWidth={1.5} />
        </svg>
      )}
    </div>
  );
}
