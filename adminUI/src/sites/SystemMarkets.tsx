import {
  NodeIndexOutlined,
  SortDescendingOutlined,
  TruckOutlined,
} from "@ant-design/icons";
import { useLazyQuery, useQuery } from "@apollo/client/react";
import {
  AutoComplete,
  Button,
  Col,
  Descriptions,
  Divider,
  Flex,
  InputNumber,
  Popover,
  Progress,
  Row,
  Segmented,
  Select,
  Space,
  Spin,
  Switch,
  Table,
} from "antd";
import { useMemo, useState } from "react";
import { Link, useParams } from "react-router-dom";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import MarketSupplyChainFlow from "../features/SupplyChainVisual/MarketSupplyChainFlow";
import WaypointLink from "../features/WaypointLink";
import {
  ActivityLevel,
  FleetType,
  GetSystemMarketsQuery,
  MarketTradeGoodType,
  NavMode,
  ShipNavStats,
  SupplyLevel,
  TradeMode,
  TradeSymbol,
} from "../gql/graphql";
import {
  GET_SYSTEM_MARKETS,
  GET_SYSTEM_TRADE_ROUTE_CANDIDATES,
} from "../graphql/queries";
import { cn, Prettify } from "../utils/utils";

function parseMultiplier(input: string): number | undefined {
  const trimmed = input.trim();
  if (trimmed === "") return undefined;
  const value = Number(trimmed);
  return Number.isFinite(value) ? value : undefined;
}

const DEFAULT_SHIP_NAV_STATS: ShipNavStats = {
  maxFuel: 100,
  maxCargo: 100,
  startRange: undefined,
  onlyMarkets: false,
  canWarp: false,
  engineSpeed: 30,
  engineCondition: 1,
  navMode: NavMode.Cruise,
};

function SystemMarkets() {
  const { systemID } = useParams();

  const [hideFuelInTradeRouteCandidates, setHideFuelInTradeRouteCandidates] =
    useState(true);
  const [hideFuelInMarketTrades, setHideFuelInMarketTrades] = useState(true);
  const [selectedShipSymbol, setSelectedShipSymbol] = useState<
    string | undefined
  >(undefined);
  const [purchaseMultiplierInput, setPurchaseMultiplierInput] = useState("");
  const [sourceMode, setSourceMode] = useState<"ship" | "stats">("ship");
  const [shipNavStats, setShipNavStats] = useState<ShipNavStats>({
    ...DEFAULT_SHIP_NAV_STATS,
  });
  const [appliedShipNavStats, setAppliedShipNavStats] = useState<ShipNavStats>({
    ...DEFAULT_SHIP_NAV_STATS,
  });

  const updateNavStats = <K extends keyof ShipNavStats>(
    key: K,
    value: ShipNavStats[K],
  ) => {
    setShipNavStats((prev) => ({ ...prev, [key]: value }));
  };

  const source = useMemo(() => {
    if (sourceMode === "ship") {
      return selectedShipSymbol ? { ship: selectedShipSymbol } : undefined;
    }
    return { shipNavStats: appliedShipNavStats };
  }, [sourceMode, selectedShipSymbol, appliedShipNavStats]);

  const { loading, error, data, refetch } = useQuery(GET_SYSTEM_MARKETS, {
    variables: {
      systemSymbol: systemID || "",
    },
  });

  const [
    loadTradeRouteCandidates,
    {
      loading: tradeRouteCandidatesLoading,
      error: tradeRouteCandidatesError,
      data: tradeRouteCandidatesData,
    },
  ] = useLazyQuery(GET_SYSTEM_TRADE_ROUTE_CANDIDATES);

  const loadTradeRouteCandidatesData = () => {
    loadTradeRouteCandidates({
      variables: {
        systemSymbol: systemID || "",
        source,
        purchaseMultiplier: parseMultiplier(purchaseMultiplierInput),
      },
    });
  };

  const marketDistripution = useMemo(() => {
    return data?.system.marketTrades.items
      .filter(
        (trade) =>
          !(
            trade.type === MarketTradeGoodType.Exchange &&
            trade.symbol === "FUEL"
          ),
      )
      .map((trade) => trade.marketTradeGood?.supply)
      .reduce(
        (a, b) => {
          if (b) {
            a[b] += 1;
            a["TOTAL"] += 1;
          }
          return a;
        },
        {
          [SupplyLevel.Scarce]: 0,
          [SupplyLevel.Limited]: 0,
          [SupplyLevel.Moderate]: 0,
          [SupplyLevel.High]: 0,
          [SupplyLevel.Abundant]: 0,
          TOTAL: 0,
        } as Record<SupplyLevel | "TOTAL", number>,
      );
  }, [data?.system.marketTrades.items]);

  const fuelCost = useMemo(() => {
    const prices = (data?.system.marketTrades.items || [])
      .filter((t) => t.symbol === "FUEL")
      .map((t) => t.marketTradeGood?.purchasePrice)
      .filter((p): p is number => p != null)
      .sort((a, b) => a - b);
    if (prices.length === 0) return null;
    const min = prices[0];
    const max = prices[prices.length - 1];
    const avg = prices.reduce((s, p) => s + p, 0) / prices.length;
    const mid = Math.floor(prices.length / 2);
    const median =
      prices.length % 2 === 0
        ? (prices[mid - 1] + prices[mid]) / 2
        : prices[mid];
    return { min, max, avg, median };
  }, [data?.system.marketTrades.items]);

  const antimatterCost = useMemo(() => {
    const prices = (data?.system.marketTrades.items || [])
      .filter((t) => t.symbol === "ANTIMATTER")
      .map((t) => t.marketTradeGood?.purchasePrice)
      .filter((p): p is number => p != null)
      .sort((a, b) => a - b);
    if (prices.length === 0) return null;
    const min = prices[0];
    const max = prices[prices.length - 1];
    const avg = prices.reduce((s, p) => s + p, 0) / prices.length;
    const mid = Math.floor(prices.length / 2);
    const median =
      prices.length % 2 === 0
        ? (prices[mid - 1] + prices[mid]) / 2
        : prices[mid];
    return { min, max, avg, median };
  }, [data?.system.marketTrades.items]);

  const marketOpportunities = useMemo(() => {
    const opportunities = (data?.system.marketTrades.items || []).reduce(
      (a, b) => {
        a[b.symbol] = a[b.symbol] || 0;
        a[b.symbol] += 1;

        return a;
      },
      {} as Record<TradeSymbol, number>,
    );

    return opportunities;
  }, [data?.system.marketTrades.items]);

  if (error) return <p>Error: {error.message}</p>;

  const tradingFleets = (data?.system.fleets.items || []).filter(
    (f) => f.fleetType === FleetType.Trading,
  ) as Prettify<
    Omit<
      GetSystemMarketsQuery["system"]["fleets"]["items"][number],
      "config"
    > & {
      config: Extract<
        GetSystemMarketsQuery["system"]["fleets"]["items"][number]["config"],
        { __typename?: "TradingConfig" }
      >;
    }
  >[];

  const tradingShips = (data?.system.ships || []).filter((s) =>
    tradingFleets.some(
      (f) => f.id === s.status.fleetId || f.id === s.status.tempFleetId,
    ),
  );

  const purchaseMultipliers = [
    ...new Set(tradingFleets.map((f) => f.config.purchaseMultiplier)),
  ].sort((a, b) => a - b);

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="System Markets" />
      <Spin spinning={loading}>
        <Space>
          <h1 className="scroll-m-20 text-center text-3xl font-bold tracking-tight text-balance">
            System Markets
          </h1>
          <Button
            onClick={() => {
              refetch();
            }}
          >
            Refresh
          </Button>
        </Space>
        <Divider plain size="small" variant="dashed" />
        <Row gutter={[8, 8]}>
          <Col span={18}>
            <Descriptions
              bordered
              column={3}
              items={[
                {
                  label: "System Symbol",
                  key: "systemSymbol",
                  children: <Link to={`/system/${systemID}`}>{systemID}</Link>,
                },
                {
                  label: "Market Trades",
                  key: "marketTrades",
                  children: data?.system.marketTrades.items.length,
                },
                {
                  label: "Market Opportunities",
                  key: "marketOpportunities",
                  children: (
                    <span>
                      {Object.entries(marketOpportunities)
                        .filter((s) => s[0] !== "FUEL")
                        .reduce((a, b) => a + (b[1] * (b[1] - 1)) / 2, 0)}{" "}
                      (
                      {Object.entries(marketOpportunities).reduce(
                        (a, b) => a + (b[1] * (b[1] - 1)) / 2,
                        0,
                      )}
                      )
                    </span>
                  ),
                },
                {
                  label: "Fuel Cost",
                  children: fuelCost ? (
                    <Flex vertical>
                      <Flex justify="space-between">
                        Min <MoneyDisplay amount={Math.ceil(fuelCost.min)} />
                      </Flex>
                      <Flex justify="space-between">
                        Max <MoneyDisplay amount={Math.ceil(fuelCost.max)} />
                      </Flex>
                      <Flex justify="space-between">
                        Avg <MoneyDisplay amount={Math.ceil(fuelCost.avg)} />
                      </Flex>
                      <Flex justify="space-between">
                        Median{" "}
                        <MoneyDisplay amount={Math.ceil(fuelCost.median)} />
                      </Flex>
                    </Flex>
                  ) : (
                    "N/A"
                  ),
                },
                {
                  label: "Antimatter Cost",
                  children: antimatterCost ? (
                    <Flex vertical>
                      <Flex justify="space-between">
                        Min{" "}
                        <MoneyDisplay amount={Math.ceil(antimatterCost.min)} />
                      </Flex>
                      <Flex justify="space-between">
                        Max{" "}
                        <MoneyDisplay amount={Math.ceil(antimatterCost.max)} />
                      </Flex>
                      <Flex justify="space-between">
                        Avg{" "}
                        <MoneyDisplay amount={Math.ceil(antimatterCost.avg)} />
                      </Flex>
                      <Flex justify="space-between">
                        Median{" "}
                        <MoneyDisplay
                          amount={Math.ceil(antimatterCost.median)}
                        />
                      </Flex>
                    </Flex>
                  ) : (
                    "N/A"
                  ),
                },
                {
                  label: "Market Distribution",
                  children: (
                    <span>
                      <div className="grid grid-cols-[max-content_max-content_auto] gap-2">
                        <span>SCARCE</span>
                        <span className="text-right">
                          {marketDistripution?.SCARCE}
                        </span>
                        <span>
                          <Progress
                            percent={
                              ((marketDistripution?.SCARCE || 0) /
                                (marketDistripution?.TOTAL || 1)) *
                              100
                            }
                            size={"small"}
                            showInfo={false}
                          />
                        </span>
                        <span>LIMITED</span>
                        <span className="text-right">
                          {marketDistripution?.LIMITED}
                        </span>
                        <span>
                          <Progress
                            percent={
                              ((marketDistripution?.LIMITED || 0) /
                                (marketDistripution?.TOTAL || 1)) *
                              100
                            }
                            size={"small"}
                            showInfo={false}
                          />
                        </span>
                        <span>MODERATE</span>
                        <span className="text-right">
                          {marketDistripution?.MODERATE}
                        </span>
                        <span>
                          <Progress
                            percent={
                              ((marketDistripution?.MODERATE || 0) /
                                (marketDistripution?.TOTAL || 1)) *
                              100
                            }
                            size={"small"}
                            showInfo={false}
                          />
                        </span>
                        <span>HIGH</span>
                        <span className="text-right">
                          {marketDistripution?.HIGH}
                        </span>
                        <span>
                          <Progress
                            percent={
                              ((marketDistripution?.HIGH || 0) /
                                (marketDistripution?.TOTAL || 1)) *
                              100
                            }
                            size={"small"}
                            showInfo={false}
                          />
                        </span>
                        <span>ABUNDANT</span>
                        <span className="text-right">
                          {marketDistripution?.ABUNDANT}
                        </span>
                        <span>
                          <Progress
                            percent={
                              ((marketDistripution?.ABUNDANT || 0) /
                                (marketDistripution?.TOTAL || 1)) *
                              100
                            }
                            size={"small"}
                            showInfo={false}
                          />
                        </span>
                      </div>
                    </span>
                  ),
                },
              ]}
            />
          </Col>
          <Col span={6}>
            <Table
              title={() => "Market Opportunities"}
              size="small"
              columns={[
                {
                  title: "Symbol",
                  dataIndex: "symbol",
                  key: "symbol",
                  sorter: (a, b) => a.symbol.localeCompare(b.symbol),
                  filters: Object.keys(marketOpportunities).map((symbol) => ({
                    text: symbol,
                    value: symbol,
                  })),
                  onFilter: (value, record) => record.symbol === value,
                },
                {
                  title: "Count",
                  dataIndex: "count",
                  key: "count",
                  align: "right",
                  sorter: (a, b) => a.count - b.count,
                },
                {
                  title: "Edges",
                  dataIndex: "edges",
                  key: "edges",
                  align: "right",
                  sorter: (a, b) => a.edges - b.edges,
                },
              ]}
              dataSource={Object.entries(marketOpportunities).map(
                ([symbol, count]) => ({
                  key: symbol,
                  symbol,
                  count,
                  edges: (count * (count - 1)) / 2,
                }),
              )}
              pagination={{
                showSizeChanger: true,
                pageSizeOptions: [
                  "10",
                  "20",
                  "50",
                  "100",
                  "200",
                  "500",
                  "1000",
                ],
                defaultPageSize: 10,
                showTotal: (total, range) =>
                  `${range[0]}-${range[1]} of ${total}`,
              }}
            />
          </Col>
        </Row>
        <Divider />
        <Row gutter={10}>
          <Col span={15}>
            <Table
              size="small"
              title={() => "Trade Fleets"}
              dataSource={tradingFleets}
              columns={[
                {
                  title: "ID",
                  dataIndex: "id",
                  key: "id",
                  sorter: (a, b) => a.id - b.id,
                },
                {
                  title: "Fleet Type",
                  dataIndex: "fleetType",
                  key: "fleetType",
                  sorter: (a, b) => a.fleetType.localeCompare(b.fleetType),
                },
                {
                  title: "Active",
                  dataIndex: "active",
                  key: "active",
                  sorter: (a, b) => (a.active ? 1 : 0) - (b.active ? 1 : 0),
                  render: (_, record) => (record.active ? "Yes" : "No"),
                },
                {
                  title: "Assignments",
                  key: "assignments",
                  align: "right",
                  sorter: (a, b) =>
                    a.assignments.items.length - b.assignments.items.length,
                  render: (_, record) => (
                    <Popover
                      title={
                        <Flex flex={1} vertical>
                          {record.assignments.items.map((asgmt) => (
                            <Flex key={asgmt.id} justify="space-between">
                              {asgmt.id} {asgmt.disabled ? "D" : "A"}|
                              <SortDescendingOutlined /> {asgmt.priority}|
                              <NodeIndexOutlined /> {asgmt.rangeMin}|
                              <TruckOutlined /> {asgmt.cargoMin}|
                              {asgmt.extractor && "E|"}
                              {asgmt.siphon && "SI|"}
                              {asgmt.survey && "SU|"}
                              {asgmt.warpDrive && "W|"}
                            </Flex>
                          ))}
                        </Flex>
                      }
                    >
                      {record.assignments.items.length}
                    </Popover>
                  ),
                },
                {
                  title: "Trade Mode",
                  key: "tradeMode",
                  sorter: (a, b) =>
                    a.config.tradeMode.localeCompare(b.config.tradeMode),
                  render: (_, record) => record.config.tradeMode,
                },
                {
                  title: "Market Blacklist",
                  key: "marketBlacklist",
                  align: "right",
                  sorter: (a, b) =>
                    a.config.marketBlacklist.length -
                    b.config.marketBlacklist.length,
                  render: (_, record) => (
                    <Popover
                      title={
                        <Flex flex={1} vertical>
                          {record.config.marketBlacklist.map((symbol) => (
                            <Flex key={symbol} justify="space-between">
                              {symbol}
                            </Flex>
                          ))}
                        </Flex>
                      }
                    >
                      {record.config.marketBlacklist.length}
                    </Popover>
                  ),
                },
                {
                  title: "Market Prefer List",
                  key: "marketPreferList",
                  align: "right",
                  sorter: (a, b) =>
                    a.config.marketPreferList.length -
                    b.config.marketPreferList.length,
                  render: (_, record) => (
                    <Popover
                      title={
                        <Flex flex={1} vertical>
                          {record.config.marketPreferList.map((symbol) => (
                            <Flex key={symbol} justify="space-between">
                              {symbol}
                            </Flex>
                          ))}
                        </Flex>
                      }
                    >
                      {record.config.marketPreferList.length}
                    </Popover>
                  ),
                },
                {
                  title: "Purchase Multiplier",
                  key: "purchaseMultiplier",
                  align: "right",
                  sorter: (a, b) =>
                    a.config.purchaseMultiplier - b.config.purchaseMultiplier,
                  render: (_, record) => record.config.purchaseMultiplier,
                },
                {
                  title: "Trade Profit Threshold",
                  key: "tradeProfitThreshold",
                  align: "right",
                  sorter: (a, b) =>
                    a.config.tradeProfitThreshold -
                    b.config.tradeProfitThreshold,
                  render: (_, record) => record.config.tradeProfitThreshold,
                },
                {
                  title: "Ship Market Ratio",
                  key: "shipMarketRatio",
                  align: "right",
                  sorter: (a, b) =>
                    a.config.shipMarketRatio - b.config.shipMarketRatio,
                  render: (_, record) => record.config.shipMarketRatio,
                },
                {
                  title: "Min Cargo Space",
                  key: "minCargoSpace",
                  align: "right",
                  sorter: (a, b) =>
                    a.config.minCargoSpace - b.config.minCargoSpace,
                  render: (_, record) => record.config.minCargoSpace,
                },
                {
                  title: "Created At",
                  dataIndex: "createdAt",
                  key: "createdAt",
                  sorter: (a, b) =>
                    new Date(a.createdAt).getTime() -
                    new Date(b.createdAt).getTime(),
                  defaultSortOrder: "descend",
                  render: (_, record) =>
                    new Date(record.createdAt).toLocaleString(),
                },
                {
                  title: "Updated At",
                  dataIndex: "updatedAt",
                  key: "updatedAt",
                  sorter: (a, b) =>
                    new Date(a.updatedAt).getTime() -
                    new Date(b.updatedAt).getTime(),
                  defaultSortOrder: "descend",
                  render: (_, record) =>
                    new Date(record.updatedAt).toLocaleString(),
                },
              ]}
            />
          </Col>
          <Col span={9}>
            <Table
              size="small"
              title={() => "Trading Ships"}
              dataSource={tradingShips}
              columns={[
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
                  title: "Waypoint",
                  dataIndex: "waypointSymbol",
                  key: "waypointSymbol",
                  render: (_, record) => (
                    <WaypointLink waypoint={record.nav.waypointSymbol}>
                      {record.nav.waypointSymbol}
                    </WaypointLink>
                  ),
                  sorter: (a, b) =>
                    a.nav.waypointSymbol.localeCompare(b.nav.waypointSymbol),
                },
                {
                  title: "Nav Status",
                  key: "navStatus",
                  render: (_, record) => record.nav.status,
                  sorter: (a, b) => a.nav.status.localeCompare(b.nav.status),
                },
                {
                  title: "Fuel",
                  key: "fuel",
                  render: (_, record) => record.fuel.capacity,
                  sorter: (a, b) => a.fuel.capacity - b.fuel.capacity,
                },
                {
                  title: "Cargo",
                  key: "cargo",
                  render: (_, record) => record.cargo.capacity,
                  sorter: (a, b) => a.cargo.capacity - b.cargo.capacity,
                },
                {
                  title: "Engine Speed",
                  key: "engineSpeed",
                  render: (_, record) => record.engineSpeed,
                  sorter: (a, b) => a.engineSpeed - b.engineSpeed,
                },
                {
                  title: "Fleet (tmp)",
                  key: "fleet",
                  render: (_, record) =>
                    `${record.status.fleetId}${record.status.tempFleetId ? ` (${record.status.tempFleetId})` : ""}`,
                  sorter: (a, b) =>
                    (a.status?.fleetId || 0) - (b.status?.fleetId || 0),
                },
                {
                  title: "Assignment (tmp)",
                  key: "assignment",
                  render: (_, record) =>
                    `${record.status.assignmentId}${record.status.tempAssignmentId ? ` (${record.status.tempAssignmentId})` : ""}`,
                  sorter: (a, b) =>
                    (a.status?.assignmentId || 0) -
                    (b.status?.assignmentId || 0),
                },
              ]}
            />
          </Col>
        </Row>
        <Divider />

        <Row gutter={10}>
          <Col span={24}>
            <MarketSupplyChainFlow
              marketTrades={(data?.system?.marketTrades.items || []).filter(
                (t) => t.symbol !== "FUEL",
              )}
            />
          </Col>
        </Row>
        <Divider />

        <Row gutter={10}>
          <Col span={12}>
            <Table
              size="small"
              title={() => (
                <Flex justify="space-between">
                  <span>Market Trades</span>
                  <Space>
                    Hide Fuel
                    <Switch
                      onChange={setHideFuelInMarketTrades}
                      checked={hideFuelInMarketTrades}
                    />
                  </Space>
                </Flex>
              )}
              columns={[
                {
                  title: "Waypoint",
                  dataIndex: "waypointSymbol",
                  key: "waypointSymbol",
                  render: (symbol: string) => (
                    <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
                  ),
                  sorter: (a, b) =>
                    a.waypointSymbol.localeCompare(b.waypointSymbol),
                  filters: [
                    ...new Set(
                      (data?.system?.marketTrades.items || []).map(
                        (t) => t.waypointSymbol,
                      ),
                    ),
                  ].map((t) => ({
                    text: t,
                    value: t,
                  })),
                  onFilter: (value, record) => record.waypointSymbol === value,
                },
                {
                  title: "Symbol",
                  dataIndex: "symbol",
                  key: "symbol",
                  sorter: (a, b) => a.symbol.localeCompare(b.symbol),
                  filters: [
                    ...new Set(
                      (data?.system?.marketTrades.items || []).map(
                        (t) => t.symbol,
                      ),
                    ),
                  ].map((t) => ({
                    text: t,
                    value: t,
                  })),
                  filterSearch: true,
                  onFilter: (value, record) => record.symbol === value,
                },
                {
                  title: "Type",
                  dataIndex: "type",
                  key: "type",
                  sorter: (a, b) => a.type.localeCompare(b.type),
                  filters: Object.values(MarketTradeGoodType).map((t) => ({
                    text: t,
                    value: t,
                  })),
                  onFilter: (value, record) => record.type === value,
                },

                {
                  title: "Supply",
                  key: "marketTradeGood?.supply",
                  render: (_, record) =>
                    record.marketTradeGood?.supply || "N/A",
                  sorter: (a, b) =>
                    (a.marketTradeGood?.supply ?? "").localeCompare(
                      b.marketTradeGood?.supply ?? "",
                    ),
                  filters: Object.values(SupplyLevel).map((t) => ({
                    text: t,
                    value: t,
                  })),
                  onFilter: (value, record) =>
                    record.marketTradeGood?.supply === value,
                },
                {
                  title: "Volume",
                  key: "marketTradeGood?.tradeVolume",
                  align: "right",
                  render: (_, record) =>
                    record.marketTradeGood?.tradeVolume || "N/A",

                  sorter: (a, b) =>
                    (a.marketTradeGood?.tradeVolume ?? 0) -
                    (b.marketTradeGood?.tradeVolume ?? 0),
                },
                {
                  title: "Activity",
                  key: "activity",
                  render: (_, record) =>
                    record.marketTradeGood?.activity || "N/A",
                  sorter: (a, b) =>
                    (a.marketTradeGood?.activity ?? "").localeCompare(
                      b.marketTradeGood?.activity ?? "",
                    ),
                  filters: Object.values(ActivityLevel).map((t) => ({
                    text: t,
                    value: t,
                  })),
                  onFilter: (value, record) =>
                    record.marketTradeGood?.activity === value,
                },
                {
                  title: "Purchase",
                  key: "marketTradeGood?.purchasePrice",
                  render: (_, record) =>
                    record.type === "IMPORT" ? (
                      record.marketTradeGood?.purchasePrice ? (
                        <MoneyDisplay
                          amount={record.marketTradeGood?.purchasePrice}
                        />
                      ) : (
                        "N/A"
                      )
                    ) : (
                      <b>
                        {record.marketTradeGood?.purchasePrice ? (
                          <MoneyDisplay
                            amount={record.marketTradeGood?.purchasePrice}
                          />
                        ) : (
                          "N/A"
                        )}
                      </b>
                    ),
                  sorter: (a, b) =>
                    (a.marketTradeGood?.purchasePrice || 0) -
                    (b.marketTradeGood?.purchasePrice || 0),
                },
                {
                  title: "Sell",
                  key: "marketTradeGood?.sellPrice",
                  render: (_, record) =>
                    record.type === "EXPORT" ? (
                      record.marketTradeGood?.sellPrice ? (
                        <MoneyDisplay
                          amount={record.marketTradeGood?.sellPrice}
                        />
                      ) : (
                        "N/A"
                      )
                    ) : (
                      <b>
                        {record.marketTradeGood?.sellPrice ? (
                          <MoneyDisplay
                            amount={record.marketTradeGood?.sellPrice}
                          />
                        ) : (
                          "N/A"
                        )}
                      </b>
                    ),
                  sorter: (a, b) =>
                    (a.marketTradeGood?.sellPrice || 0) -
                    (b.marketTradeGood?.sellPrice || 0),
                },
                {
                  title: "Created At",
                  dataIndex: "createdAt",
                  key: "createdAt",
                  render: (date: string) => new Date(date).toLocaleString(),
                  sorter: (a, b) =>
                    new Date(a.createdAt).getTime() -
                    new Date(b.createdAt).getTime(),
                },
              ]}
              dataSource={(data?.system?.marketTrades.items || []).filter(
                (t) => !hideFuelInMarketTrades || t.symbol !== "FUEL",
              )}
              rowKey={(row) => row.symbol + row.waypointSymbol + row.type}
              pagination={{
                showTotal: (total, range) =>
                  `${range[0]}-${range[1]} of ${total}`,
              }}
            />
          </Col>
          <Col span={12}>
            <Table
              size="small"
              columns={[
                // {
                //   title: "ID",
                //   dataIndex: "id",
                //   key: "id",
                //   sorter: (a, b) => a.id - b.id,
                // },
                {
                  title: "Created At",
                  dataIndex: "createdAt",
                  key: "createdAt",
                  render: (date: string, record) => (
                    <Popover content={<span>{record.id}</span>}>
                      <span>{new Date(date).toLocaleString()}</span>
                    </Popover>
                  ),
                  sorter: (a, b) =>
                    new Date(a.createdAt).getTime() -
                    new Date(b.createdAt).getTime(),
                  defaultSortOrder: "descend",
                },
                {
                  title: "Trade Symbol",
                  dataIndex: "symbol",
                  key: "symbol",
                  sorter: (a, b) => a.symbol.localeCompare(b.symbol),
                  filters: Object.values(TradeSymbol).map((sym) => ({
                    text: sym,
                    value: sym,
                  })),
                  onFilter: (value, record) => record.symbol === value,
                },
                {
                  title: "Ship",
                  dataIndex: "shipSymbol",
                  key: "shipSymbol",
                  render: (symbol: string) => (
                    <Link to={`/ships/${symbol}`}>{symbol}</Link>
                  ),
                  sorter: (a, b) => a.shipSymbol.localeCompare(b.shipSymbol),
                },
                {
                  title: "Purchase WP",
                  dataIndex: "PurchaseWaypointSymbol",
                  key: "PurchaseWaypointSymbol",
                  render: (symbol: string) => (
                    <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
                  ),
                  sorter: (a, b) =>
                    a.PurchaseWaypointSymbol.localeCompare(
                      b.PurchaseWaypointSymbol,
                    ),
                },
                {
                  title: "Sell WP",
                  dataIndex: "SellWaypointSymbol",
                  key: "SellWaypointSymbol",
                  render: (symbol: string) => (
                    <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
                  ),
                  sorter: (a, b) =>
                    a.SellWaypointSymbol.localeCompare(b.SellWaypointSymbol),
                },
                {
                  title: "Status",
                  dataIndex: "status",
                  key: "status",
                  sorter: (a, b) => a.status.localeCompare(b.status),
                  filters: [
                    { text: "Delivered", value: "DELIVERED" },
                    { text: "Failed", value: "FAILED" },
                    { text: "In Transit", value: "IN_TRANSIT" },
                  ],
                  onFilter: (value, record) => record.status === value,
                },
                {
                  title: "Trade Mode",
                  dataIndex: "tradeMode",
                  key: "tradeMode",
                  sorter: (a, b) => a.tradeMode.localeCompare(b.tradeMode),
                  filters: Object.values(TradeMode).map((mode) => ({
                    text: mode,
                    value: mode,
                  })),
                  onFilter: (value, record) => record.tradeMode === value,
                },
                {
                  title: "Volume",
                  dataIndex: "tradeVolume",
                  key: "tradeVolume",
                  align: "right",
                  sorter: (a, b) => a.tradeVolume - b.tradeVolume,
                },
                {
                  title: "Expenses",
                  key: "expenses",
                  align: "right",
                  render: (_, record) => (
                    <span>
                      <MoneyDisplay
                        amount={
                          record.marketTransactionSummary?.allExpenses || 0
                        }
                      />
                    </span>
                  ),
                  sorter: (a, b) =>
                    (a.marketTransactionSummary?.allExpenses || 0) -
                    (b.marketTransactionSummary?.allExpenses || 0),
                },
                {
                  title: "Profit",
                  key: "profit",
                  align: "right",
                  render: (_, record) => (
                    <Popover
                      content={
                        <Flex flex={1} vertical>
                          <span className="font-bold">Prediction</span>
                          <Flex justify="space-between" gap={10}>
                            <span>Income:</span>{" "}
                            <MoneyDisplay
                              amount={
                                (record.sellMarketTradeGood?.sellPrice || 0) *
                                  record.tradeVolume || 0
                              }
                            />
                          </Flex>
                          <Flex justify="space-between" gap={10}>
                            <span>Expenses:</span>{" "}
                            <MoneyDisplay
                              amount={
                                (record.purchaseMarketTradeGood
                                  ?.purchasePrice || 0) * record.tradeVolume ||
                                0
                              }
                            />
                          </Flex>
                          <Flex justify="space-between" gap={10}>
                            <span>Fuel:</span>{" "}
                            <MoneyDisplay amount={record.estimatedFuel || 0} />
                          </Flex>
                          <Flex justify="space-between" gap={10}>
                            <span>Profit:</span>{" "}
                            <MoneyDisplay
                              amount={
                                ((record.sellMarketTradeGood?.sellPrice || 0) *
                                  record.tradeVolume || 0) -
                                ((record.purchaseMarketTradeGood
                                  ?.purchasePrice || 0) * record.tradeVolume ||
                                  0) -
                                (record.estimatedFuel || 0)
                              }
                            />
                          </Flex>
                          <span className="font-bold">Summary</span>
                          <Flex justify="space-between" gap={10}>
                            <span>Income:</span>{" "}
                            <MoneyDisplay
                              amount={
                                record.marketTransactionSummary?.allIncome || 0
                              }
                            />
                          </Flex>
                          <Flex justify="space-between" gap={10}>
                            <span>Expenses:</span>{" "}
                            <MoneyDisplay
                              amount={
                                record.marketTransactionSummary?.allExpenses ||
                                0
                              }
                            />
                          </Flex>
                          <Flex justify="space-between" gap={10}>
                            <span>Profit:</span>{" "}
                            <MoneyDisplay
                              amount={
                                (record.marketTransactionSummary?.allIncome ||
                                  0) -
                                (record.marketTransactionSummary?.allExpenses ||
                                  0)
                              }
                            />
                          </Flex>
                        </Flex>
                      }
                    >
                      <MoneyDisplay
                        amount={
                          (record.marketTransactionSummary?.allIncome || 0) -
                          (record.marketTransactionSummary?.allExpenses || 0)
                        }
                        className={cn(
                          (record.marketTransactionSummary?.allIncome || 0) -
                            (record.marketTransactionSummary?.allExpenses ||
                              0) >
                            0
                            ? "text-current"
                            : "text-red-600",
                        )}
                      />
                    </Popover>
                  ),
                  sorter: (a, b) =>
                    (a.marketTransactionSummary?.allIncome || 0) -
                    (a.marketTransactionSummary?.allExpenses || 0) -
                    (b.marketTransactionSummary?.allIncome || 0) +
                    (b.marketTransactionSummary?.allExpenses || 0),
                },
              ]}
              dataSource={data?.system.tradeRoutes.items || []}
              rowKey={(row) => row.id.toString()}
              title={() => "Trade Routes"}
              pagination={{
                showSizeChanger: true,
                pageSizeOptions: ["10", "20", "50", "100"],
                defaultPageSize: 10,
                showTotal: (total, range) =>
                  `${range[0]}-${range[1]} of ${total}`,
              }}
            />
          </Col>
        </Row>
        <Divider />

        <Row>
          <Col span={24}>
            <Table
              loading={tradeRouteCandidatesLoading}
              columns={[
                {
                  title: "Symbol",
                  dataIndex: "symbol",
                  key: "symbol",
                  sorter: (a, b) => a.symbol.localeCompare(b.symbol),
                  filters: [
                    ...new Set(
                      (
                        tradeRouteCandidatesData?.system.tradeRouteCandidates
                          .items || []
                      ).map((candidate) => candidate.symbol),
                    ),
                  ].map((symbol) => ({
                    text: symbol,
                    value: symbol,
                  })),
                  filterSearch: true,
                  onFilter: (value, record) => record.symbol === value,
                },
                {
                  title: "Purchase WP",
                  key: "purchaseWaypointSymbol",
                  render: (_, record) =>
                    record.purchase.waypointSymbol || "N/A",
                  sorter: (a, b) =>
                    a.purchase.waypointSymbol.localeCompare(
                      b.purchase.waypointSymbol,
                    ),
                },
                {
                  title: "Sell WP",
                  key: "sellWaypointSymbol",
                  render: (_, record) => record.sell.waypointSymbol || "N/A",
                  sorter: (a, b) =>
                    a.sell.waypointSymbol.localeCompare(b.sell.waypointSymbol),
                },
                {
                  title: "Purchase Type",
                  key: "purchaseType",
                  render: (_, record) => record.purchase.type,
                  sorter: (a, b) =>
                    a.purchase.type.localeCompare(b.purchase.type),
                  filters: Object.values(MarketTradeGoodType).map((t) => ({
                    text: t,
                    value: t,
                  })),
                  onFilter: (value, record) => record.purchase.type === value,
                },
                {
                  title: "Sell Type",
                  key: "sellType",
                  render: (_, record) => record.sell.type,
                  sorter: (a, b) => a.sell.type.localeCompare(b.sell.type),
                  filters: Object.values(MarketTradeGoodType).map((t) => ({
                    text: t,
                    value: t,
                  })),
                  onFilter: (value, record) => record.sell.type === value,
                },
                {
                  title: "Purchase Price",
                  key: "purchasePrice",
                  align: "right",
                  render: (_, record) =>
                    record.tradeRouteProposal?.purchaseGood ? (
                      <MoneyDisplay
                        amount={
                          record.tradeRouteProposal.purchaseGood.purchasePrice
                        }
                      />
                    ) : (
                      "N/A"
                    ),
                  sorter: (a, b) =>
                    (a.tradeRouteProposal?.purchaseGood?.purchasePrice ?? 0) -
                    (b.tradeRouteProposal?.purchaseGood?.purchasePrice ?? 0),
                },
                {
                  title: "Sell Price",
                  key: "sellPrice",
                  align: "right",
                  render: (_, record) =>
                    record.tradeRouteProposal?.sellGood ? (
                      <MoneyDisplay
                        amount={record.tradeRouteProposal.sellGood.sellPrice}
                      />
                    ) : (
                      "N/A"
                    ),
                  sorter: (a, b) =>
                    (a.tradeRouteProposal?.sellGood?.sellPrice ?? 0) -
                    (b.tradeRouteProposal?.sellGood?.sellPrice ?? 0),
                },
                {
                  title: "Buy Supply",
                  key: "purchaseSupply",
                  render: (_, record) =>
                    record.tradeRouteProposal?.purchaseGood?.supply || "N/A",
                  sorter: (a, b) =>
                    (
                      a.tradeRouteProposal?.purchaseGood?.supply ?? ""
                    ).localeCompare(
                      b.tradeRouteProposal?.purchaseGood?.supply ?? "",
                    ),
                },
                {
                  title: "Sell Supply",
                  key: "sellSupply",
                  render: (_, record) =>
                    record.tradeRouteProposal?.sellGood?.supply || "N/A",
                  sorter: (a, b) =>
                    (
                      a.tradeRouteProposal?.sellGood?.supply ?? ""
                    ).localeCompare(
                      b.tradeRouteProposal?.sellGood?.supply ?? "",
                    ),
                },
                {
                  title: "Buy Activity",
                  key: "purchaseActivity",
                  render: (_, record) =>
                    record.tradeRouteProposal?.purchaseGood?.activity || "N/A",
                  sorter: (a, b) =>
                    (
                      a.tradeRouteProposal?.purchaseGood?.activity ?? ""
                    ).localeCompare(
                      b.tradeRouteProposal?.purchaseGood?.activity ?? "",
                    ),
                },
                {
                  title: "Sell Activity",
                  key: "sellActivity",
                  render: (_, record) =>
                    record.tradeRouteProposal?.sellGood?.activity || "N/A",
                  sorter: (a, b) =>
                    (
                      a.tradeRouteProposal?.sellGood?.activity ?? ""
                    ).localeCompare(
                      b.tradeRouteProposal?.sellGood?.activity ?? "",
                    ),
                },
                {
                  title: "Trade Volume",
                  key: "tradeVolume",
                  align: "right",
                  render: (_, record) =>
                    record.tradeRouteProposal?.tradeVolume ?? "N/A",
                  sorter: (a, b) =>
                    (a.tradeRouteProposal?.tradeVolume ?? 0) -
                    (b.tradeRouteProposal?.tradeVolume ?? 0),
                },
                {
                  title: "Good Cost",
                  key: "goodCost",
                  align: "right",
                  render: (_, record) =>
                    record.tradeRouteProposal ? (
                      <MoneyDisplay
                        amount={record.tradeRouteProposal.goodCost}
                      />
                    ) : (
                      "N/A"
                    ),
                  sorter: (a, b) =>
                    (a.tradeRouteProposal?.goodCost ?? 0) -
                    (b.tradeRouteProposal?.goodCost ?? 0),
                },
                {
                  title: "Good Total Sell",
                  key: "goodTotalSellPrice",
                  align: "right",
                  render: (_, record) =>
                    record.tradeRouteProposal ? (
                      <MoneyDisplay
                        amount={record.tradeRouteProposal.goodTotalSellPrice}
                      />
                    ) : (
                      "N/A"
                    ),
                  sorter: (a, b) =>
                    (a.tradeRouteProposal?.goodTotalSellPrice ?? 0) -
                    (b.tradeRouteProposal?.goodTotalSellPrice ?? 0),
                },
                {
                  title: "Good Profit",
                  key: "goodProfit",
                  align: "right",
                  render: (_, record) =>
                    record.tradeRouteProposal ? (
                      <MoneyDisplay
                        amount={record.tradeRouteProposal.goodProfit}
                      />
                    ) : (
                      "N/A"
                    ),
                  sorter: (a, b) =>
                    (a.tradeRouteProposal?.goodProfit ?? 0) -
                    (b.tradeRouteProposal?.goodProfit ?? 0),
                },
                {
                  title: "Travel Cost",
                  key: "travelCost",
                  align: "right",
                  render: (_, record) =>
                    record.tradeRouteProposal ? (
                      <MoneyDisplay
                        amount={record.tradeRouteProposal.travelCost}
                      />
                    ) : (
                      "N/A"
                    ),
                  sorter: (a, b) =>
                    (a.tradeRouteProposal?.travelCost ?? 0) -
                    (b.tradeRouteProposal?.travelCost ?? 0),
                },
                {
                  title: "Total Cost",
                  key: "totalCost",
                  align: "right",
                  render: (_, record) =>
                    record.tradeRouteProposal ? (
                      <MoneyDisplay
                        amount={record.tradeRouteProposal.totalCost}
                      />
                    ) : (
                      "N/A"
                    ),
                  sorter: (a, b) =>
                    (a.tradeRouteProposal?.totalCost ?? 0) -
                    (b.tradeRouteProposal?.totalCost ?? 0),
                },
                {
                  title: "Total Profit",
                  key: "totalProfit",
                  align: "right",
                  render: (_, record) =>
                    record.tradeRouteProposal ? (
                      <MoneyDisplay
                        amount={record.tradeRouteProposal.totalProfit}
                      />
                    ) : (
                      "N/A"
                    ),
                  sorter: (a, b) =>
                    (a.tradeRouteProposal?.totalProfit ?? 0) -
                    (b.tradeRouteProposal?.totalProfit ?? 0),
                },
                {
                  title: "Fuel Units",
                  key: "fuelUnits",
                  align: "right",
                  render: (_, record) =>
                    record.tradeRouteProposal?.fuelUnits ?? "N/A",
                  sorter: (a, b) =>
                    (a.tradeRouteProposal?.fuelUnits ?? 0) -
                    (b.tradeRouteProposal?.fuelUnits ?? 0),
                },
                {
                  title: "Time",
                  key: "time",
                  align: "right",
                  render: (_, record) =>
                    record.tradeRouteProposal?.time != null
                      ? record.tradeRouteProposal.time.toFixed(2)
                      : "N/A",
                  sorter: (a, b) =>
                    (a.tradeRouteProposal?.time ?? 0) -
                    (b.tradeRouteProposal?.time ?? 0),
                },
                {
                  title: "Distance",
                  key: "distance",
                  align: "right",
                  render: (_, record) =>
                    record.tradeRouteProposal?.distance != null
                      ? record.tradeRouteProposal.distance.toFixed(2)
                      : "N/A",
                  sorter: (a, b) =>
                    (a.tradeRouteProposal?.distance ?? 0) -
                    (b.tradeRouteProposal?.distance ?? 0),
                },
                {
                  title: "API Requests",
                  key: "apiRequests",
                  align: "right",
                  render: (_, record) =>
                    record.tradeRouteProposal?.apiRequests ?? "N/A",
                  sorter: (a, b) =>
                    (a.tradeRouteProposal?.apiRequests ?? 0) -
                    (b.tradeRouteProposal?.apiRequests ?? 0),
                },
                {
                  title: "Trips/Hour",
                  key: "tripsPerHour",
                  align: "right",
                  render: (_, record) =>
                    record.tradeRouteProposal?.tripsPerHour != null
                      ? record.tradeRouteProposal.tripsPerHour.toFixed(2)
                      : "N/A",
                  sorter: (a, b) =>
                    (a.tradeRouteProposal?.tripsPerHour ?? 0) -
                    (b.tradeRouteProposal?.tripsPerHour ?? 0),
                },
                {
                  title: "Profit/Hour",
                  key: "profitPerHour",
                  align: "right",
                  render: (_, record) =>
                    record.tradeRouteProposal?.profitPerHour != null
                      ? record.tradeRouteProposal.profitPerHour.toFixed(2)
                      : "N/A",
                  sorter: (a, b) =>
                    (a.tradeRouteProposal?.profitPerHour ?? 0) -
                    (b.tradeRouteProposal?.profitPerHour ?? 0),
                },
                {
                  title: "Profit/API",
                  key: "profitPerApiRequest",
                  align: "right",
                  render: (_, record) =>
                    record.tradeRouteProposal?.profitPerApiRequest != null
                      ? record.tradeRouteProposal.profitPerApiRequest.toFixed(2)
                      : "N/A",
                  sorter: (a, b) =>
                    (a.tradeRouteProposal?.profitPerApiRequest ?? 0) -
                    (b.tradeRouteProposal?.profitPerApiRequest ?? 0),
                },
                {
                  title: "Same Waypoint",
                  key: "sameWaypoint",
                  align: "right",
                  render: (_, record) =>
                    record.purchase.waypointSymbol ===
                    record.sell.waypointSymbol
                      ? "yes"
                      : "no",
                  sorter: (a, b) =>
                    (a.purchase.waypointSymbol === a.sell.waypointSymbol
                      ? 1
                      : 0) -
                    (b.purchase.waypointSymbol === b.sell.waypointSymbol
                      ? 1
                      : 0),
                  filters: [
                    {
                      text: "Yes",
                      value: "yes",
                    },
                    {
                      text: "No",
                      value: "no",
                    },
                  ],
                  onFilter: (value, record) =>
                    (record.purchase.waypointSymbol ===
                    record.sell.waypointSymbol
                      ? "yes"
                      : "no") === value,
                  defaultFilteredValue: ["no"],
                },
              ]}
              scroll={{ x: "max-content" }}
              rowKey={(row) =>
                row.symbol +
                row.purchase.waypointSymbol +
                row.sell.waypointSymbol
              }
              dataSource={(
                tradeRouteCandidatesData?.system.tradeRouteCandidates.items ||
                []
              ).filter(
                (candidate) =>
                  !hideFuelInTradeRouteCandidates ||
                  candidate.symbol !== "FUEL",
              )}
              pagination={{
                showSizeChanger: true,
                pageSizeOptions: [
                  "10",
                  "20",
                  "50",
                  "100",
                  "200",
                  "500",
                  "1000",
                ],
                defaultPageSize: 10,
                showTotal: (total, range) =>
                  `${range[0]}-${range[1]} of ${total}`,
              }}
              title={() => (
                <Flex justify="space-between" gap={8} wrap>
                  <Flex vertical gap={4}>
                    <span>Trade Route Candidates</span>
                    {tradeRouteCandidatesError && (
                      <span className="text-red-600">
                        {tradeRouteCandidatesError.message}
                      </span>
                    )}
                  </Flex>
                  <Flex vertical gap={4} align="flex-end">
                    <Space wrap>
                      <Segmented
                        size="small"
                        value={sourceMode}
                        onChange={(value) =>
                          setSourceMode(value as "ship" | "stats")
                        }
                        options={[
                          { label: "Ship", value: "ship" },
                          { label: "Stats", value: "stats" },
                        ]}
                      />
                      {sourceMode === "ship" && (
                        <Select
                          allowClear
                          showSearch
                          placeholder="Default stats"
                          style={{ minWidth: 180 }}
                          value={selectedShipSymbol}
                          onChange={(value) => setSelectedShipSymbol(value)}
                          options={tradingShips.map((s) => ({
                            label: s.symbol,
                            value: s.symbol,
                          }))}
                        />
                      )}
                      <span>Multiplier</span>
                      <AutoComplete
                        allowClear
                        placeholder="Auto"
                        style={{ width: 120 }}
                        value={purchaseMultiplierInput}
                        onChange={(value) =>
                          setPurchaseMultiplierInput(value ?? "")
                        }
                        options={purchaseMultipliers.map((m) => ({
                          label: String(m),
                          value: String(m),
                        }))}
                      />
                      <span>Hide Fuel</span>
                      <Switch
                        onChange={setHideFuelInTradeRouteCandidates}
                        checked={hideFuelInTradeRouteCandidates}
                      />
                      <Button
                        size="small"
                        type="primary"
                        onClick={loadTradeRouteCandidatesData}
                      >
                        {tradeRouteCandidatesData ? "Refresh" : "Load"}
                      </Button>
                    </Space>
                    {sourceMode === "stats" && (
                      <Space wrap size={4}>
                        <InputNumber
                          size="small"
                          addonBefore="Fuel"
                          value={shipNavStats.maxFuel}
                          onChange={(value) =>
                            updateNavStats("maxFuel", value ?? 0)
                          }
                        />
                        <InputNumber
                          size="small"
                          addonBefore="Cargo"
                          value={shipNavStats.maxCargo}
                          onChange={(value) =>
                            updateNavStats("maxCargo", value ?? 0)
                          }
                        />
                        <InputNumber
                          size="small"
                          addonBefore="Range"
                          value={shipNavStats.startRange ?? undefined}
                          onChange={(value) =>
                            updateNavStats("startRange", value ?? undefined)
                          }
                        />
                        <InputNumber
                          size="small"
                          addonBefore="Speed"
                          value={shipNavStats.engineSpeed}
                          onChange={(value) =>
                            updateNavStats("engineSpeed", value ?? 0)
                          }
                        />
                        <InputNumber
                          size="small"
                          addonBefore="Cond"
                          step={0.1}
                          value={shipNavStats.engineCondition}
                          onChange={(value) =>
                            updateNavStats("engineCondition", value ?? 0)
                          }
                        />
                        <Select
                          size="small"
                          style={{ width: 170 }}
                          value={shipNavStats.navMode}
                          onChange={(value) => updateNavStats("navMode", value)}
                          options={Object.values(NavMode).map((m) => ({
                            label: m,
                            value: m,
                          }))}
                        />
                        <span>Markets</span>
                        <Switch
                          size="small"
                          checked={shipNavStats.onlyMarkets}
                          onChange={(checked) =>
                            updateNavStats("onlyMarkets", checked)
                          }
                        />
                        <span>Warp</span>
                        <Switch
                          size="small"
                          checked={shipNavStats.canWarp}
                          onChange={(checked) =>
                            updateNavStats("canWarp", checked)
                          }
                        />
                        <Button
                          size="small"
                          type="primary"
                          onClick={() => setAppliedShipNavStats(shipNavStats)}
                        >
                          Apply
                        </Button>
                      </Space>
                    )}
                  </Flex>
                </Flex>
              )}
            />
          </Col>
        </Row>
      </Spin>
    </div>
  );
}

export default SystemMarkets;
