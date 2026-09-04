import { Flex, Popover, Table, TableProps } from "antd";
import {
  GetSystemQuery,
  TradeSymbol,
  WaypointModifierSymbol,
  WaypointTraitSymbol,
  WaypointType,
} from "../../gql/graphql";
import MarketTradeGoodsPopover from "../MarketTradeGoods/MarketTradeGoodsPopover";
import MoneyDisplay from "../MonyDisplay";
import Timer from "../Timer/Timer";
import WaypointLink from "../WaypointLink";

type GQLWaypoint = GetSystemQuery["system"]["waypoints"]["items"][number];

function WaypointTable({
  waypoints,
  ...props
}: {
  waypoints: GQLWaypoint[];
} & TableProps<GQLWaypoint>) {
  const systemSymbol = waypoints[0]?.systemSymbol || "";

  const columns: TableProps<GQLWaypoint>["columns"] = [
    {
      title: "Symbol",
      dataIndex: "symbol",
      key: "symbol",
      render: (symbol: string) => (
        <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
      ),
      sorter: (a, b) => a.symbol.localeCompare(b.symbol),
    },
    {
      title: "Type",
      dataIndex: "waypointType",
      key: "waypointType",
      sorter: (a, b) => a.waypointType.localeCompare(b.waypointType),
      filters: Object.values(WaypointType).map((type) => ({
        text: type,
        value: type,
      })),
      onFilter: (value, record) => record.waypointType === value,
    },
    {
      title: "Pos X",
      dataIndex: "x",
      key: "x",
      sorter: (a, b) => a.x - b.x,
    },
    {
      title: "Pos Y",
      dataIndex: "y",
      key: "y",
      sorter: (a, b) => a.y - b.y,
    },
    {
      title: "Orbitals",
      dataIndex: "orbitals",
      key: "orbitals",
      render: (orbitals: string[]) =>
        orbitals.length > 0 ? (
          <Flex gap={1} vertical>
            {orbitals.map((symbol) => (
              <WaypointLink waypoint={symbol} key={symbol}>
                {symbol.replace(systemSymbol + "-", "")}
              </WaypointLink>
            ))}
          </Flex>
        ) : (
          "None"
        ), // List symbols of orbitals or "None"
      sorter: (a, b) => a.orbitals.length - b.orbitals.length,
    },
    {
      title: "Orbits",
      dataIndex: "orbits",
      key: "orbits",
      render: (orbits: string) =>
        orbits ? (
          <WaypointLink waypoint={orbits}>
            {orbits.replace(systemSymbol + "-", "")}
          </WaypointLink>
        ) : (
          "N/A"
        ), // Display "N/A" if undefined
      sorter: (a, b) => (a.orbits ?? "").localeCompare(b.orbits ?? ""),
    },
    {
      title: "Traits",
      dataIndex: "traits",
      key: "traits",
      render: (traits) => (
        <Popover
          title={
            <Flex gap={1} vertical>
              {traits.map((trait: WaypointTraitSymbol) => (
                <span key={trait}>{trait}</span>
              ))}
            </Flex>
          }
        >
          {traits.length}
        </Popover>
      ), // List names of traits
      sorter: (a, b) => a.traits.length - b.traits.length,
      filters: Object.values(WaypointTraitSymbol).map((trait) => ({
        text: trait,
        value: trait,
      })),
      onFilter: (value, record) => record.traits.some((t) => t === value),
    },
    {
      title: "Modifiers",
      dataIndex: "modifiers",
      key: "modifiers",
      render: (modifiers) =>
        modifiers && modifiers.length > 0 ? (
          <span>
            {modifiers?.map((modifier: WaypointModifierSymbol) => (
              <span key={modifier}>{modifier}</span>
            ))}
          </span>
        ) : (
          "None"
        ),
      sorter: (a, b) => (a.modifiers?.length ?? 0) - (b.modifiers?.length ?? 0),
      filters: Object.values(WaypointModifierSymbol).map((modifier) => ({
        text: modifier,
        value: modifier,
      })),
      onFilter: (value, record) =>
        record.modifiers?.some((m) => m === value) ?? false,
    },
    {
      title: "Scrap",
      key: "nextScrap",
      render: (_, record) =>
        record.nextScrap && record.lastScrap ? (
          <Popover
            title={
              <span>
                {new Date(record.lastScrap).toLocaleString()} -{" "}
                {Math.floor(
                  (new Date(record.nextScrap).getTime() -
                    new Date(record.lastScrap).getTime()) /
                    1000 /
                    60,
                )}
                min{" "}
                {Math.floor(
                  ((new Date(record.nextScrap).getTime() -
                    new Date(record.lastScrap).getTime()) /
                    1000) %
                    60,
                )}
                s - {new Date(record.nextScrap).toLocaleString()}
              </span>
            }
          >
            T <Timer time={record.nextScrap} />
          </Popover>
        ) : (
          "N/A"
        ), // Display chart symbol or "N/A"
      sorter: (a, b, sortOrder) =>
        (a.nextScrap ?? (sortOrder == "ascend" ? "9" : "0")).localeCompare(
          b.nextScrap ?? (sortOrder == "ascend" ? "9" : "0"),
        ),
    },
    {
      title: "Has Market",
      dataIndex: "hasMarketplace",
      key: "hasMarketplace",
      render: (value) => (value ? "Yes" : "No"), // Render boolean as "Yes" or "No"
      sorter: (a, b) => (a.hasMarketplace ? 1 : 0) - (b.hasMarketplace ? 1 : 0),
      filters: [
        { text: "Yes", value: true },
        { text: "No", value: false },
      ],
      onFilter: (value, record) => record.hasMarketplace === value,
    },
    {
      title: "Trade Goods",
      dataIndex: "marketTrades",
      key: "marketTrades",
      render: (marketTrades: GQLWaypoint["marketTrades"]) =>
        marketTrades && marketTrades.items.length > 0 ? (
          <>
            <Popover
              content={
                <MarketTradeGoodsPopover marketTrades={marketTrades.items} />
              }
            >
              <Flex gap={1} flex={1} vertical>
                {marketTrades.items.filter((t) => t.type === "EXCHANGE")
                  .length > 0 && (
                  <Flex justify="space-between">
                    <span>EXCHANGE</span>
                    <span>
                      {
                        marketTrades.items.filter((t) => t.type === "EXCHANGE")
                          .length
                      }
                    </span>
                  </Flex>
                )}
                {marketTrades.items.filter((t) => t.type === "IMPORT").length >
                  0 && (
                  <Flex justify="space-between">
                    <span>IMPORT</span>
                    <span>
                      {
                        marketTrades.items.filter((t) => t.type === "IMPORT")
                          .length
                      }
                    </span>
                  </Flex>
                )}
                {marketTrades.items.filter((t) => t.type === "EXPORT").length >
                  0 && (
                  <Flex justify="space-between">
                    <span>EXPORT</span>
                    <span>
                      {
                        marketTrades.items.filter((t) => t.type === "EXPORT")
                          .length
                      }
                    </span>
                  </Flex>
                )}
              </Flex>
            </Popover>
          </>
        ) : (
          "None"
        ),
      sorter: (a, b) =>
        (a.marketTrades?.items.length ?? 0) -
        (b.marketTrades?.items.length ?? 0),
      filters: Object.values(TradeSymbol).map((trade_good) => ({
        text: trade_good,
        value: trade_good,
      })),
      filterSearch: true,
      onFilter: (value, record) =>
        record.marketTrades?.items.some((t) => t.symbol === value) ?? false,
    },
    {
      title: "Has Shipyard",
      dataIndex: "hasShipyard",
      key: "hasShipyard",
      render: (value) => (value ? "Yes" : "No"), // Render boolean as "Yes" or "No"
      sorter: (a, b) => (a.hasShipyard ? 1 : 0) - (b.hasShipyard ? 1 : 0),
      filters: [
        { text: "Yes", value: true },
        { text: "No", value: false },
      ],
      onFilter: (value, record) => record.hasShipyard === value,
    },
    {
      title: "Shipyard Ships",
      dataIndex: "shipyardShips",
      key: "shipyardShips",
      render: (value: GQLWaypoint["shipyardShips"]) => (
        <Popover
          title={
            <Flex gap={1} vertical>
              {value.items.map((ship) => (
                <Flex justify="space-between" gap={4}>
                  <span>{ship.shipType}</span>{" "}
                  <span>
                    <MoneyDisplay amount={ship.purchasePrice} />{" "}
                    <span className="font-mono">{ship.supply.slice(0, 3)}</span>
                  </span>
                </Flex>
              ))}
            </Flex>
          }
        >
          Ships {value.items.length}
        </Popover>
      ),
      sorter: (a, b) =>
        (a.shipyardShips.items.length ?? 0) -
        (b.shipyardShips.items.length ?? 0),
      filters: [
        ...[
          ...new Set(
            waypoints.map((w) => w?.shipyardShips?.items ?? []).flat() ?? [],
          ),
        ]
          .map((sh) => sh.shipType)
          .map((sh) => ({
            text: sh,
            value: sh,
          })),
      ],
      onFilter: (value, record) => record.hasShipyard === value,
    },
    {
      title: "Construction",
      dataIndex: "isUnderConstruction",
      key: "isUnderConstruction",
      render: (value) => (value ? "Yes" : "No"), // Render boolean as "Yes" or "No"
      sorter: (a, b) =>
        (a.isUnderConstruction ? 1 : 0) - (b.isUnderConstruction ? 1 : 0),
      filters: [
        { text: "Yes", value: true },
        { text: "No", value: false },
      ],
      onFilter: (value, record) => record.isUnderConstruction === value,
    },
    {
      title: "Charted",
      key: "charted",
      render: (_, record) =>
        record.chartedBy && record.chartedOn ? (
          <Popover
            title={
              <span>
                {record.chartedBy}
                <br />
                {new Date(record.chartedOn).toLocaleString()}
              </span>
            }
          >
            {record.chartedBy.split("-")[0]}
          </Popover>
        ) : (
          "N/A"
        ), // Display chart symbol or "N/A"
      sorter: (a, b) => (a.chartedBy ?? "").localeCompare(b.chartedBy ?? ""),
    },
    {
      title: "Faction",
      dataIndex: "faction",
      key: "faction",
      render: (faction) => (faction ? faction : "N/A"), // Display faction symbol or "N/A"
      sorter: (a, b) => (a.faction ?? "").localeCompare(b.faction ?? ""),
    },
  ];

  return (
    <Table
      size="middle"
      columns={columns}
      title={() => "Waypoints"}
      dataSource={waypoints || []}
      rowKey={(row) => row.symbol}
      pagination={{
        showTotal: (total, range) => `${range[0]}-${range[1]} of ${total}`,
      }}
      {...props}
    />
  );
}

export default WaypointTable;
