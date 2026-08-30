import { theme } from "antd";
import {
  PolarAngleAxis,
  PolarGrid,
  PolarRadiusAxis,
  Radar,
  RadarChart,
  ResponsiveContainer,
  Tooltip,
} from "recharts";
import { ShipModuleSymbol, ShipMountSymbol } from "../../gql/graphql";
import { ShipData } from "../../sites/Ship";

function mountMiningScore(m: ShipMountSymbol): number {
  switch (m) {
    case ShipMountSymbol.MiningLaserI:
    case ShipMountSymbol.GasSiphonI:
    case ShipMountSymbol.SurveyorI:
      return 1;
    case ShipMountSymbol.MiningLaserIi:
    case ShipMountSymbol.GasSiphonIi:
    case ShipMountSymbol.SurveyorIi:
      return 2;
    case ShipMountSymbol.MiningLaserIii:
    case ShipMountSymbol.GasSiphonIii:
    case ShipMountSymbol.SurveyorIii:
      return 3;
    default:
      return 0;
  }
}

function mountSensorScore(m: ShipMountSymbol): number {
  switch (m) {
    case ShipMountSymbol.SensorArrayI:
      return 1;
    case ShipMountSymbol.SensorArrayIi:
      return 2;
    case ShipMountSymbol.SensorArrayIii:
      return 3;
    default:
      return 0;
  }
}

function moduleWarpScore(m: ShipModuleSymbol): number {
  switch (m) {
    case ShipModuleSymbol.WarpDriveI:
      return 1;
    case ShipModuleSymbol.WarpDriveIi:
      return 2;
    case ShipModuleSymbol.WarpDriveIii:
      return 3;
    default:
      return 0;
  }
}

function moduleRefineryScore(m: ShipModuleSymbol): number {
  switch (m) {
    case ShipModuleSymbol.FuelRefineryI:
    case ShipModuleSymbol.MicroRefineryI:
    case ShipModuleSymbol.OreRefineryI:
      return 1;
    default:
      return 0;
  }
}

const RADAR_MAX: Record<string, number> = {
  Cargo: 490,
  Fuel: 4000,
  Engine: 60,
  Mining: 9,
  Sensors: 3,
  Warp: 3,
  Refinery: 3,
  Condition: 1,
};

const CARGO_TIERS = [0, 15, 40, 80, 150, 225, 490];
const FUEL_TIERS = [0, 80, 160, 300, 400, 600, 800, 2300, 4000, 4080];

// Map a value to a 0-100 score using the discrete tier list so each possible
// step is visible. Because the tiers grow multiplicatively, spacing them evenly
// gives a logarithmic feel without crushing the smaller values.
function tierScore(value: number, tiers: number[]): number {
  if (value <= 0) return 0;
  let idx = 0;
  for (let i = 0; i < tiers.length; i++) {
    if (value >= tiers[i]) idx = i;
  }
  return (idx / (tiers.length - 1)) * 100;
}

function radarScore(capability: string, value: number): number {
  switch (capability) {
    case "Cargo":
      return tierScore(value, CARGO_TIERS);
    case "Fuel":
      return tierScore(value == 0 ? 4080 : value, FUEL_TIERS);
    default:
      return (value / RADAR_MAX[capability]) * 100;
  }
}

type RadarTooltipProps = {
  active?: boolean;
  payload?: Array<{
    dataKey?: string | number;
    payload: { Capability: string; value: number; max: number };
  }>;
  label?: string;
};

function RadarTooltip({ active, payload, label }: RadarTooltipProps) {
  const {
    token: {
      colorBgElevated,
      colorBorderSecondary,
      colorTextSecondary,
      borderRadiusLG,
      boxShadowSecondary,
    },
  } = theme.useToken();

  if (!active || !payload?.length) return null;

  const entry = payload.find((p) => p.dataKey === "score");
  if (!entry) return null;

  const { value, max } = entry.payload;

  return (
    <div
      style={{
        background: colorBgElevated,
        border: `1px solid ${colorBorderSecondary}`,
        borderRadius: borderRadiusLG,
        padding: "8px 12px",
        boxShadow: boxShadowSecondary,
      }}
    >
      <div style={{ color: colorTextSecondary, fontSize: 12 }}>{label}</div>
      <div style={{ fontWeight: 600 }}>
        {value.toLocaleString()}{" "}
        <span style={{ color: colorTextSecondary, fontWeight: 400 }}>
          / {max.toLocaleString()}
        </span>
      </div>
    </div>
  );
}

function ShipCapabilityRadar({ ship }: { ship: ShipData }) {
  const {
    token: { colorText, colorPrimary, colorBorder },
  } = theme.useToken();

  const radarData = [
    {
      Capability: "Condition",
      value:
        ship.conditions.engine.condition *
        ship.conditions.frame.condition *
        ship.conditions.reactor.condition *
        ship.conditions.engine.integrity *
        ship.conditions.frame.integrity *
        ship.conditions.reactor.integrity,
    },
    { Capability: "Fuel", value: ship.fuel.capacity },
    { Capability: "Engine", value: ship.engineSpeed },
    {
      Capability: "Mining",
      value: ship.mounts.mounts.reduce((a, m) => a + mountMiningScore(m), 0),
    },
    {
      Capability: "Sensors",
      value: ship.mounts.mounts.reduce((a, m) => a + mountSensorScore(m), 0),
    },

    {
      Capability: "Warp",
      value: ship.modules.modules.reduce((a, m) => a + moduleWarpScore(m), 0),
    },
    {
      Capability: "Refinery",
      value: ship.modules.modules.reduce(
        (a, m) => a + moduleRefineryScore(m),
        0,
      ),
    },
    { Capability: "Cargo", value: ship.cargo.capacity },
  ].map((d) => ({
    ...d,
    max: RADAR_MAX[d.Capability],
    maxScore: 100,
    score: Math.round(radarScore(d.Capability, d.value) * 10) / 10,
  }));

  return (
    <ResponsiveContainer width="100%" height="100%">
      <RadarChart data={radarData} cx="50%" cy="50%" outerRadius="80%">
        <defs>
          <linearGradient id="radarFill" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor={colorPrimary} stopOpacity={0.65} />
            <stop offset="100%" stopColor={colorPrimary} stopOpacity={0.15} />
          </linearGradient>
        </defs>
        <PolarGrid stroke={colorBorder} />
        <PolarAngleAxis
          dataKey="Capability"
          tick={{ fill: colorText, fontSize: 12 }}
          tickLine={false}
        />
        <PolarRadiusAxis
          angle={90}
          domain={[0, 100]}
          tick={false}
          axisLine={false}
        />
        <Radar
          name={ship.symbol}
          dataKey="score"
          stroke={colorPrimary}
          fill="url(#radarFill)"
          strokeWidth={2}
          dot={{ r: 3, fill: colorPrimary, strokeWidth: 0 }}
          activeDot={{ r: 5, fill: colorPrimary, strokeWidth: 0 }}
        />
        <Radar
          name="Max"
          dataKey="maxScore"
          stroke={colorBorder}
          fill="none"
          strokeDasharray="4 4"
          dot={false}
        />
        <Tooltip content={<RadarTooltip />} />
      </RadarChart>
    </ResponsiveContainer>
  );
}

export default ShipCapabilityRadar;
