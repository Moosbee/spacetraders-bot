import {
  Col,
  Descriptions,
  Divider,
  Flex,
  Popover,
  Row,
  Table,
  TableProps,
  theme,
} from "antd";
import { ResponsiveContainer, Tooltip, Treemap } from "recharts";
import { ShipModuleSymbol, TradeSymbol } from "../../gql/graphql";
import { ShipData } from "../../sites/Ship";

const REQUIREMENT_COLORS: Record<string, string> = {
  Power: "#ff7300",
  Crew: "#8884d8",
  Slots: "#82ca9d",
};

const LEFTOVER_COLOR = "#bfbfbf";

function RequirementTreemapCell({
  x = 0,
  y = 0,
  width = 0,
  height = 0,
  depth = 0,
  name = "",
  fill = "#8884d8",
  value = 0,
}: {
  x?: number;
  y?: number;
  width?: number;
  height?: number;
  depth?: number;
  name?: string;
  fill?: string;
  value?: number;
}) {
  if (width <= 0 || height <= 0) return null;
  const showLabel = depth > 0 && width > 40 && height > 10;
  const canShowTwoLines = depth > 0 && width > 40 && height > 24;
  return (
    <g>
      <rect
        x={x}
        y={y}
        width={width}
        height={height}
        fill={fill}
        stroke="#fff"
        strokeWidth={2 / (depth + 1)}
      />
      {showLabel ? (
        <foreignObject x={x} y={y} width={width} height={height}>
          <div
            style={{
              width: "100%",
              height: "100%",
              display: "flex",
              justifyContent: "center",
              alignItems: "center",
              color: "#fff",
              fontSize: 11,
            }}
          >
            <div className="text-center wrap-break-word">
              {name.split("_").map((item, i) => (
                <>
                  {i > 0 && (
                    <>
                      _<wbr />
                    </>
                  )}
                  {item}
                </>
              ))}
              {canShowTwoLines ? <br /> : " "}
              {value}
            </div>
          </div>
        </foreignObject>
      ) : null}
    </g>
  );
}

function ShipComponents({ ship }: { ship: ShipData }) {
  const {
    token: { colorBgElevated },
  } = theme.useToken();

  const mountColumns: TableProps<
    ShipData["mounts"]["mountInfos"][number]
  >["columns"] = [
    {
      title: "Type",
      dataIndex: "symbol",
      key: "symbol",
      sorter: (a, b) => a.symbol.localeCompare(b.symbol),
    },
    {
      title: "Strength",
      dataIndex: "strength",
      key: "strength",
      align: "right",
      sorter: (a, b) => (a.strength ?? 0) - (b.strength ?? 0),
      render: (value) => value ?? "N/A",
    },
    {
      title: "Deposits",
      dataIndex: "deposits",
      key: "deposits",
      render: (value?: TradeSymbol[]) =>
        value ? (
          <Popover
            title={
              <Flex gap={2} vertical>
                {value.map((d) => (
                  <span key={d}>{d}</span>
                ))}
              </Flex>
            }
          >
            <span className="flex flex-row items-center justify-between flex-nowrap">
              <span>
                {value
                  .map((d) =>
                    d
                      .split("_")
                      .map((s) => s.charAt(0).toUpperCase())
                      .join(""),
                  )
                  .join(", ")}
              </span>
              <span>{value.length}</span>
            </span>
          </Popover>
        ) : (
          "N/A"
        ),
    },
    {
      title: "Power",
      dataIndex: "powerRequired",
      key: "powerRequired",
      align: "right",
      render: (value) => value ?? "N/A",
    },
    {
      title: "Crew",
      dataIndex: "crewRequired",
      key: "crewRequired",
      align: "right",
      render: (value) => value ?? "N/A",
    },
    {
      title: "Slots",
      dataIndex: "slotsRequired",
      key: "slotsRequired",
      align: "right",
      render: (value) => value ?? 1,
    },
  ];

  const moduleColumns: TableProps<
    ShipData["modules"]["moduleInfos"][number]
  >["columns"] = [
    {
      title: "Type",
      dataIndex: "symbol",
      key: "symbol",
      sorter: (a, b) => a.symbol.localeCompare(b.symbol),
    },
    {
      title: "Capacity",
      dataIndex: "capacity",
      key: "capacity",
      align: "right",
      render: (value) => value ?? "N/A",
    },
    {
      title: "Range",
      dataIndex: "range",
      key: "range",
      align: "right",
      render: (value) => value ?? "N/A",
    },
    {
      title: "Power",
      dataIndex: "powerRequired",
      key: "powerRequired",
      align: "right",
      render: (value) => value ?? "N/A",
    },
    {
      title: "Crew",
      dataIndex: "crewRequired",
      key: "crewRequired",
      align: "right",
      render: (value) => value ?? "N/A",
    },
    {
      title: "Slots",
      dataIndex: "slotsRequired",
      key: "slotsRequired",
      align: "right",
      render: (value) => value ?? "N/A",
    },
  ];

  const requirements = [
    ...[
      {
        symbol: ship.engineInfo.symbol,
        power: ship.engineInfo.powerRequired ?? 0,
        crew: ship.engineInfo.crewRequired ?? 0,
        slots: ship.engineInfo.slotsRequired ?? 0,
      },
      {
        symbol: ship.reactorInfo.symbol,
        power: ship.reactorInfo.powerRequired ?? 0,
        crew: ship.reactorInfo.crewRequired ?? 0,
        slots: ship.reactorInfo.slotsRequired ?? 0,
      },
      {
        symbol: ship.frameInfo.symbol,
        power: ship.frameInfo.powerRequired ?? 0,
        crew: ship.frameInfo.crewRequired ?? 0,
        slots: ship.frameInfo.slotsRequired ?? 0,
      },
    ],
    ...ship.mounts.mountInfos.map((mount) => {
      return {
        symbol: mount.symbol,
        power: mount.powerRequired ?? 0,
        crew: mount.crewRequired ?? 0,
        slots: mount.slotsRequired ?? 0,
      };
    }),
    ...ship.modules.moduleInfos.map((module) => {
      return {
        symbol: module.symbol,
        power: module.powerRequired ?? 0,
        crew: module.crewRequired ?? 0,
        slots: module.slotsRequired ?? 0,
      };
    }),
  ];

  const totalPower = requirements.reduce((a, b) => a + b.power, 0);
  const totalCrew = requirements.reduce((a, b) => a + b.crew, 0);
  const totalSlots = requirements.reduce((a, b) => a + b.slots, 0);

  const crewCapacity = ship.modules.moduleInfos
    .filter((m) => m.symbol === ShipModuleSymbol.CrewQuartersI)
    .reduce((a, m) => a + (m.capacity || 0), 0);

  const powerRequirements = [
    ...requirements.map((r) => ({ name: r.symbol, size: r.power })),
    {
      name: "Leftover",
      size: Math.max(0, (ship.reactorInfo.powerOutput ?? 0) - totalPower),
      fill: LEFTOVER_COLOR,
    },
  ];
  const crewRequirements = [
    ...requirements.map((r) => ({ name: r.symbol, size: r.crew })),
    {
      name: "Leftover",
      size: Math.max(0, crewCapacity - totalCrew),
      fill: LEFTOVER_COLOR,
    },
  ];
  const slotsRequirements = [
    ...requirements.map((r) => ({ name: r.symbol, size: r.slots })),
    {
      name: "Leftover",
      size: Math.max(0, ship.frameInfo.moduleSlots - totalSlots),
      fill: LEFTOVER_COLOR,
    },
  ];

  return (
    <>
      <Row gutter={[8, 8]}>
        <Col span={5}>
          <Descriptions
            bordered
            size="small"
            column={2}
            title="Frame"
            items={[
              {
                label: "Symbol",
                children: ship.frameInfo.symbol,
              },
              {
                label: "Fuel Capacity",
                children: (
                  <span className="text-nowrap">
                    {ship.frameInfo.fuelCapacity}
                  </span>
                ),
              },
              {
                label: "Module Slots",
                children: ship.frameInfo.moduleSlots,
              },
              {
                label: "Mount Points",
                children: ship.frameInfo.mountingPoints,
              },
              {
                label: "Condition",
                children: ship.conditions.frame.condition,
              },
              {
                label: "Integrity",
                children: ship.conditions.frame.integrity,
              },
              {
                label: "Required Power",
                children: ship.frameInfo.powerRequired,
              },
              {
                label: "Required Crew",
                children: ship.frameInfo.crewRequired,
              },
            ]}
          />
        </Col>
        <Col span={3}>
          <Descriptions
            bordered
            size="small"
            layout="vertical"
            column={2}
            title="Reactor"
            items={[
              {
                label: "Symbol",
                children: ship.reactorInfo.symbol,
              },
              {
                label: "Power Output",
                children: ship.reactorInfo.powerOutput,
              },
              {
                label: "Condition",
                children: ship.conditions.reactor.condition,
              },
              {
                label: "Integrity",
                children: ship.conditions.reactor.integrity,
              },
              {
                label: "Required Crew",
                children: ship.reactorInfo.crewRequired,
              },
            ]}
          />
        </Col>
        <Col span={4}>
          <Descriptions
            bordered
            size="small"
            layout="vertical"
            column={2}
            title="Engine"
            items={[
              {
                label: "Symbol",
                children: ship.engineInfo.symbol,
              },
              {
                label: "Speed",
                children: ship.engineInfo.speed,
              },
              {
                label: "Condition",
                children: ship.conditions.engine.condition,
              },
              {
                label: "Integrity",
                children: ship.conditions.engine.integrity,
              },
              {
                label: "Required Crew",
                children: ship.engineInfo.crewRequired,
              },
              {
                label: "Required Power",
                children: ship.engineInfo.powerRequired,
              },
            ]}
          />
        </Col>
        <Col span={6}>
          <Table
            size="small"
            rowKey={(record, i) => `${record.symbol}-${i}`}
            title={() =>
              `Mounts ${ship.mounts.mounts.length} (${ship.mounts.mountInfos.map((m) => m.slotsRequired ?? 1).reduce((a, b) => a + b, 0)}/${ship.frameInfo.mountingPoints})`
            }
            pagination={false}
            columns={mountColumns}
            dataSource={ship.mounts.mountInfos}
            locale={{ emptyText: <span>No mounts</span> }}
          />
        </Col>
        <Col span={6}>
          <Table
            size="small"
            title={() =>
              `Modules ${ship.modules.modules.length} (${ship.modules.moduleInfos.map((m) => m.slotsRequired ?? 1).reduce((a, b) => a + b, 0)}/${ship.frameInfo.moduleSlots})`
            }
            pagination={false}
            columns={moduleColumns}
            dataSource={ship.modules.moduleInfos}
            rowKey={(record, i) => record.symbol + i}
            locale={{ emptyText: <span>No modules</span> }}
          />
        </Col>
      </Row>
      <Divider />
      <Row gutter={[8, 8]}>
        <Col span={5}>
          <Table
            // bordered
            pagination={false}
            size="small"
            columns={[
              { title: "Symbol", dataIndex: "symbol" },
              {
                title: "Power",
                dataIndex: "power",
                align: "right",
                sorter: (a, b) => (a.power ?? 0) - (b.power ?? 0),
              },
              {
                title: "Crew",
                dataIndex: "crew",
                align: "right",
                sorter: (a, b) => (a.crew ?? 0) - (b.crew ?? 0),
              },
              {
                title: "Slots",
                dataIndex: "slots",
                align: "right",
                sorter: (a, b) => (a.slots ?? 0) - (b.slots ?? 0),
              },
            ]}
            dataSource={requirements}
          />
        </Col>
        <Col span={4}>
          <Table
            // bordered
            pagination={false}
            size="small"
            columns={[
              { title: "Category", dataIndex: "category" },
              {
                title: "Total Req",
                dataIndex: "totalRequired",
                align: "right",
              },
              {
                title: "Total Av",
                dataIndex: "totalAvailable",
                align: "right",
              },
            ]}
            dataSource={[
              {
                category: "Crew",
                totalRequired: requirements
                  .map((r) => Math.max(r.crew ?? 0, 0))
                  .reduce((a, b) => a + b, 0),
                totalAvailable:
                  ship.modules.moduleInfos
                    .filter((m) => m.symbol === ShipModuleSymbol.CrewQuartersI)
                    .map((m) => m.capacity || 0)
                    .reduce((a, b) => a + b, 0) -
                  Math.min(ship.frameInfo.crewRequired ?? 0, 0),
              },
              {
                category: "Power",
                totalRequired: requirements
                  .map((r) => r.power || 0)
                  .reduce((a, b) => a + b, 0),
                totalAvailable: ship.reactorInfo.powerOutput ?? 0,
              },
              {
                category: "Module Slots",
                totalRequired: requirements
                  .map((r) => r.slots || 0)
                  .reduce((a, b) => a + b, 0),
                totalAvailable: ship.frameInfo.moduleSlots,
              },
              {
                category: "Mount Slots",
                totalRequired: ship.mounts.mountInfos
                  .map((m) => m.slotsRequired ?? 1)
                  .reduce((a, b) => a + b, 0),
                totalAvailable: ship.frameInfo.mountingPoints,
              },
            ]}
          />
        </Col>
        <Col span={5}>
          <div style={{ fontWeight: 600, marginBottom: 4 }}>Crew</div>
          <div className="w-full min-h-80 h-11/12">
            <ResponsiveContainer width="100%" height="100%">
              <Treemap
                data={crewRequirements}
                dataKey="size"
                content={
                  <RequirementTreemapCell fill={REQUIREMENT_COLORS.Crew} />
                }
              >
                <Tooltip
                  content={(props) => {
                    if (
                      !props.payload ||
                      !props.payload.length ||
                      !props.payload[0].payload
                    )
                      return null;
                    return (
                      <div
                        style={{
                          whiteSpace: "pre-wrap",
                          background: colorBgElevated,
                          padding: "0.5rem",
                        }}
                      >
                        <div>{props.payload?.[0].payload?.name}</div>
                        <div>{props.payload?.[0].payload?.value}</div>
                      </div>
                    );
                  }}
                />
              </Treemap>
            </ResponsiveContainer>
          </div>
        </Col>
        <Col span={5}>
          <div style={{ fontWeight: 600, marginBottom: 4 }}>Power</div>
          <div className="w-full min-h-80 h-11/12">
            <ResponsiveContainer width="100%" height="100%">
              <Treemap
                data={powerRequirements}
                dataKey="size"
                content={
                  <RequirementTreemapCell fill={REQUIREMENT_COLORS.Power} />
                }
              >
                <Tooltip
                  content={(props) => {
                    if (
                      !props.payload ||
                      !props.payload.length ||
                      !props.payload[0].payload
                    )
                      return null;
                    return (
                      <div
                        style={{
                          whiteSpace: "pre-wrap",
                          background: colorBgElevated,
                          padding: "0.5rem",
                        }}
                      >
                        <div>{props.payload?.[0].payload?.name}</div>
                        <div>{props.payload?.[0].payload?.value}</div>
                      </div>
                    );
                  }}
                />
              </Treemap>
            </ResponsiveContainer>
          </div>
        </Col>

        <Col span={5}>
          <div style={{ fontWeight: 600, marginBottom: 4 }}>Slots</div>
          <div className="w-full min-h-80 h-11/12">
            <ResponsiveContainer width="100%" height="100%">
              <Treemap
                data={slotsRequirements}
                dataKey="size"
                content={
                  <RequirementTreemapCell fill={REQUIREMENT_COLORS.Slots} />
                }
              >
                <Tooltip
                  content={(props) => {
                    if (
                      !props.payload ||
                      !props.payload.length ||
                      !props.payload[0].payload
                    )
                      return null;
                    return (
                      <div
                        style={{
                          whiteSpace: "pre-wrap",
                          background: colorBgElevated,
                          padding: "0.5rem",
                        }}
                      >
                        <div>{props.payload?.[0].payload?.name}</div>
                        <div>{props.payload?.[0].payload?.value}</div>
                      </div>
                    );
                  }}
                />
              </Treemap>
            </ResponsiveContainer>
          </div>
        </Col>
      </Row>
    </>
  );
}

export default ShipComponents;
