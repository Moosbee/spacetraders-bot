import { useMemo } from "react";
import {
  Bar,
  BarChart,
  CartesianGrid,
  ErrorBar,
  Rectangle,
  RectangleProps,
  ReferenceDot,
  ResponsiveContainer,
  Tooltip,
  TooltipProps,
  XAxis,
  YAxis,
} from "recharts";

export interface BoxPlotDatum {
  label: string;
  values: number[];
}

interface BoxStats {
  label: string;
  count: number;
  min: number;
  max: number;
  q1: number;
  median: number;
  q3: number;
  whiskerLow: number;
  whiskerHigh: number;
  outliers: number[];
}

interface BoxPlotProps {
  data: BoxPlotDatum[];
  title?: string;
  color?: string;
}

function quantile(sorted: number[], q: number): number {
  if (sorted.length === 0) return 0;
  if (sorted.length === 1) return sorted[0];
  const pos = (sorted.length - 1) * q;
  const base = Math.floor(pos);
  const rest = pos - base;
  if (base + 1 < sorted.length) {
    return sorted[base] + rest * (sorted[base + 1] - sorted[base]);
  }
  return sorted[base];
}

function computeStats(data: BoxPlotDatum[]): BoxStats[] {
  return data
    .filter((d) => d.values.length > 0)
    .map((d) => {
      const sorted = [...d.values].sort((a, b) => a - b);
      const q1 = quantile(sorted, 0.25);
      const median = quantile(sorted, 0.5);
      const q3 = quantile(sorted, 0.75);
      const iqr = q3 - q1;
      const lowerFence = q1 - 1.5 * iqr;
      const upperFence = q3 + 1.5 * iqr;
      const whiskerLow = sorted.find((v) => v >= lowerFence) ?? sorted[0];
      const whiskerHigh =
        [...sorted].reverse().find((v) => v <= upperFence) ??
        sorted[sorted.length - 1];
      const outliers = sorted.filter((v) => v < lowerFence || v > upperFence);
      return {
        label: d.label,
        count: d.values.length,
        min: sorted[0],
        max: sorted[sorted.length - 1],
        q1,
        median,
        q3,
        whiskerLow,
        whiskerHigh,
        outliers,
      };
    });
}

// Recharts ranges the bar between the two values returned by `dataKey`.
const boxDataKey = (entry: BoxStats): [number, number] => [entry.q1, entry.q3];

// ErrorBar is anchored at the top of the box (q3): the first value is the
// distance down to the lower whisker, the second up to the upper whisker.
const whiskerDataKey = (entry: BoxStats): [number, number] => [
  entry.q3 - entry.whiskerLow,
  entry.whiskerHigh - entry.q3,
];

interface BoxShapeProps {
  x?: number;
  y?: number;
  width?: number;
  height?: number;
  q1: number;
  median: number;
  q3: number;
}

const BoxShape = (props: unknown): JSX.Element => {
  const {
    x = 0,
    y = 0,
    width = 0,
    height = 0,
    q1,
    median,
    q3,
  } = props as BoxShapeProps;
  const quartileRange = q3 - q1;
  const medianOffset =
    quartileRange === 0 ? width / 2 : ((median - q1) / quartileRange) * width;
  const medianX = x + medianOffset;

  return (
    <g>
      <Rectangle {...(props as RectangleProps)} />
      <line
        x1={medianX}
        x2={medianX}
        y1={y}
        y2={y + height}
        stroke="currentColor"
        strokeWidth={2}
      />
    </g>
  );
};

const renderTooltip = ({ active, payload }: TooltipProps<number, string>) => {
  if (!active || !payload || payload.length === 0) {
    return null;
  }
  const entry = payload[0]?.payload as BoxStats | undefined;
  if (!entry) {
    return null;
  }
  return (
    <div
      style={{
        background: "rgba(0, 0, 0, 0.85)",
        color: "#fff",
        padding: "8px 12px",
        borderRadius: 6,
        fontSize: 12,
        lineHeight: 1.5,
      }}
    >
      <div style={{ fontWeight: 600 }}>{entry.label}</div>
      <div>count: {entry.count}</div>
      <div>min: {entry.min.toLocaleString()}</div>
      <div>Q1: {entry.q1.toLocaleString()}</div>
      <div>median: {entry.median.toLocaleString()}</div>
      <div>Q3: {entry.q3.toLocaleString()}</div>
      <div>max: {entry.max.toLocaleString()}</div>
      {entry.outliers.length > 0 && (
        <div>outliers: {entry.outliers.length}</div>
      )}
    </div>
  );
};

export default function BoxPlot({
  data,
  title,
  color = "#1677ff",
}: BoxPlotProps) {
  const stats = useMemo(() => computeStats(data), [data]);

  const domain = useMemo(() => {
    const values = data.flatMap((d) => d.values);
    if (values.length === 0) return null;
    const min = Math.min(...values);
    const max = Math.max(...values);
    const pad = (max - min) * 0.05 || Math.abs(max) * 0.05 || 1;
    return [min - pad, max + pad] as [number, number];
  }, [data]);

  if (stats.length === 0 || !domain) {
    return (
      <div>
        {title && <h3>{title}</h3>}
        <p>No data</p>
      </div>
    );
  }

  const outliers = stats.flatMap((s) =>
    s.outliers.map((value) => ({ label: s.label, value })),
  );

  const chartHeight = Math.max(300, stats.length * 30 + 100);
  const maxLabelLength = Math.max(...stats.map((s) => s.label.length), 1);
  const yAxisWidth = Math.min(220, Math.max(60, maxLabelLength * 7 + 16));

  return (
    <div>
      {title && <h3 style={{ margin: "0 0 8px" }}>{title}</h3>}
      <div style={{ overflowX: "auto" }}>
        <ResponsiveContainer width="100%" height={chartHeight}>
          <BarChart
            data={stats}
            layout="vertical"
            margin={{ top: 10, right: 40, bottom: 10, left: 0 }}
          >
            <CartesianGrid horizontal={false} strokeDasharray="3 3" />
            <XAxis
              type="number"
              domain={domain}
              tickFormatter={(value: number) => value.toLocaleString()}
            />
            <YAxis
              type="category"
              dataKey="label"
              width={yAxisWidth}
              interval={0}
            />
            <Tooltip content={renderTooltip} />
            <Bar
              dataKey={boxDataKey}
              shape={BoxShape}
              fill={color}
              fillOpacity={0.35}
              stroke={color}
              strokeWidth={1}
              isAnimationActive={false}
            >
              <ErrorBar
                dataKey={whiskerDataKey}
                width={10}
                stroke={color}
                strokeWidth={1}
              />
            </Bar>
            {outliers.map((o, i) => (
              <ReferenceDot
                key={`${o.label}-${i}`}
                x={o.value}
                y={o.label}
                r={3}
                fill="none"
                stroke={color}
                strokeWidth={1}
                ifOverflow="extendDomain"
              />
            ))}
          </BarChart>
        </ResponsiveContainer>
      </div>
      <div style={{ fontSize: 12, opacity: 0.7 }}>
        Box: Q1-Q3 · line: median · whiskers: 1.5xIQR · dots: outliers
      </div>
    </div>
  );
}
