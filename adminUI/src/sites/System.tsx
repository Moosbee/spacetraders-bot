import { useMutation, useQuery } from "@apollo/client/react";
import {
  Alert,
  Button,
  Card,
  Col,
  Descriptions,
  DescriptionsProps,
  Divider,
  Flex,
  List,
  Popover,
  Progress,
  Row,
  Space,
  Switch,
  Table,
} from "antd";
import { useMemo, useState } from "react";
import { Link, useParams } from "react-router-dom";
import AssignmentsPopover from "../features/AssignmentsPopover/AssignmentsPopover";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import WaypointLink from "../features/WaypointLink";
import WaypointTable from "../features/WaypointTable/WaypointTable";
import {
  ActivityLevel,
  MarketTradeGoodType,
  ShipType,
  SupplyLevel,
  TradeMode,
  TradeSymbol,
} from "../gql/graphql";
import { REPOPULATE_SYSTEMS_WITH_FLEETS_FROM_SYSTEM } from "../graphql/mutations";
import { GET_SYSTEM } from "../graphql/queries";
import { useAppDispatch, useAppSelector } from "../redux/hooks";
import {
  selectSelectedSystemSymbol,
  setSelectedSystemSymbol,
} from "../redux/slices/mapSlice";
import { message } from "../utils/antdMessage";
import { cn } from "../utils/utils";
import { systemIcons } from "../utils/waypointColors";

function System() {
  const { systemID } = useParams();

  const { loading, error, data, dataState, refetch } = useQuery(GET_SYSTEM, {
    variables: { systemSymbol: systemID || "" },
  });

  const [
    repopulateSystem,
    { loading: repopulateSystemLoading, error: repopulateSystemError },
  ] = useMutation(REPOPULATE_SYSTEMS_WITH_FLEETS_FROM_SYSTEM, {
    refetchQueries: [GET_SYSTEM],
  });

  const selectedSystem = useAppSelector(selectSelectedSystemSymbol);

  const dispatch = useAppDispatch();

  // if (dataState != "complete") return <p>Loading...</p>;

  const system = data?.system
    ? {
        ...data.system,
        fleets: data.system.fleets.items.map((fleet) => ({
          ...fleet,
          assignments: fleet.assignments.items,
        })),
        chartTransactions: data.system.chartTransactions.items,
        shipyardShips: data.system.shipyardShips.items,
        constructionMaterials: data.system.constructionMaterials.items,
        jumpGateConnections: data.system.jumpGateConnections.items,
        shipyardTransactions: data.system.shipyardTransactions.items,
        contractDeliveries: data.system.contractDeliveries.items,
        tradeRoutes: data.system.tradeRoutes.items,
      }
    : undefined;

  const [hideFuelInMarketTrades, setHideFuelInMarketTrades] = useState(true);

  const marketDistripution = useMemo(() => {
    return system?.marketTrades.items
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
  }, [system?.marketTrades]);

  const marketOpportunities = useMemo(() => {
    const opportunities = (system?.marketTrades.items || []).reduce(
      (a, b) => {
        a[b.symbol] = a[b.symbol] || 0;
        a[b.symbol] += 1;

        return a;
      },
      {} as Record<TradeSymbol, number>,
    );

    return opportunities;
  }, [system?.marketTrades]);

  const fuelCost = useMemo(() => {
    const prices = (system?.marketTrades.items || [])
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
  }, [system?.marketTrades]);

  const antimatterCost = useMemo(() => {
    const prices = (system?.marketTrades.items || [])
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
  }, [system?.marketTrades]);

  if (error) return <p>Error: {error.message}</p>;

  const color = systemIcons[system?.systemType || "BLACK_HOLE"].color;
  const systemIcon = systemIcons[system?.systemType || "BLACK_HOLE"].icon;

  const items: DescriptionsProps["items"] = [
    {
      label: "Symbol",
      key: "symbol",
      children: (
        <button
          onClick={() => {
            if (selectedSystem === system?.symbol) {
              dispatch(setSelectedSystemSymbol(undefined));
              return;
            }
            dispatch(setSelectedSystemSymbol(system?.symbol));
          }}
          className="cursor-pointer flex justify-between w-full py-2"
        >
          <div
            style={{
              color: color,
            }}
            className="h-6 w-6 flex justify-center items-center text-xl"
          >
            <span
              className="absolute"
              style={{
                boxShadow:
                  selectedSystem == system?.symbol
                    ? "0px 0px calc(0.8 * 1.25rem) calc(0.6 * 1.25rem) color-mix(in srgb, currentColor 80%, #fff 20%)"
                    : "",
              }}
            ></span>
            {systemIcon}
          </div>
          {system?.symbol}
        </button>
      ),
    },
    {
      key: "constellation",
      label: "Constellation",
      children: system?.constellation || "N/A",
    },
    {
      span: "filled",
      key: "reload",
      label: <Link to={`/map/system/${systemID}`}>Map</Link>,
      children: (
        <span className="flex justify-evenly items-center">
          <Button
            loading={loading || dataState !== "complete"}
            onClick={() => {
              refetch();
            }}
          >
            Reload
          </Button>
        </span>
      ),
    },
    {
      label: "Sector Symbol",
      key: "sectorSymbol",
      children: system?.sectorSymbol,
    },
    {
      label: "System Type",
      key: "systemType",
      children: system?.systemType,
    },
    {
      label: "Fleets",
      key: "Fleets",
      children: (
        <span>
          {system?.fleets.length}{" "}
          <Button
            loading={repopulateSystemLoading}
            onClick={() => {
              repopulateSystem({
                variables: {
                  systemSymbol: systemID || "",
                },
              }).then(() => {
                message.success(`Repopulated system ${systemID} with fleets!`);
                refetch();
              });
            }}
          >
            Repopulate
          </Button>
        </span>
      ),
    },
    {
      label: "Population Disabled",
      key: "populationDisabled",
      children: system?.populationDisabled ? "Yes" : "No",
    },
    {
      label: "X Coordinate",
      key: "x",
      children: system?.x,
    },
    {
      label: "Y Coordinate",
      key: "y",
      children: system?.y,
    },
    {
      label: "Waypoints",
      key: "waypoints",
      children: `${system?.waypoints.items.filter((wp) => wp.chartedOn).length}/${
        system?.waypoints.items.length
      }`,
    },
    {
      label: "Marketplaces",
      key: "marketplaces",
      children: `${
        system?.waypoints.items
          .filter((wp) => wp.hasMarketplace)
          .filter((wp) => wp.chartedOn).length
      }/${system?.waypoints.items.filter((wp) => wp.hasMarketplace).length}`,
    },
    {
      label: "Shipyards",
      key: "shipyards",
      children: `${
        system?.waypoints.items
          .filter((wp) => wp.hasShipyard)
          .filter((wp) => wp.chartedOn).length
      }/${system?.waypoints.items.filter((wp) => wp.hasShipyard).length}`,
    },
    {
      label: "Unique Trade Goods",
      key: "uniqueTradeSymbols",
      children: `${Object.keys(marketOpportunities).length}`,
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
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title={`System ${systemID}`} />
      <Space>
        <h2>System {systemID}</h2>
        {repopulateSystemError && (
          <Alert message={repopulateSystemError.message} type="error" />
        )}
        <Link to={`/system/${systemID}/markets`}>Markets</Link>
      </Space>
      <br />

      <Space>
        <Descriptions bordered column={3} items={items} />

        <Card size="small" title="Known Agents">
          <List
            size="small"
            dataSource={[...(system?.seenAgents || [])].sort(
              (a, b) => b.count - a.count,
            )}
            renderItem={(agent) => (
              <List.Item>
                <Link to={`/agents/${agent.symbol}`}>
                  {agent.symbol} ({agent.count})
                </Link>
              </List.Item>
            )}
          />
        </Card>

        <Card
          size="small"
          title={`Ships in System (${system?.ships?.length || 0})`}
        >
          <List
            size="small"
            style={{ maxHeight: "200px", overflowY: "auto" }}
            dataSource={system?.ships.toSorted((a, b) =>
              a.symbol.localeCompare(b.symbol),
            )}
            renderItem={(ship) => (
              <List.Item>
                <Popover
                  title={
                    <Flex flex={1}>
                      {ship.symbol} {ship.fuel.capacity} {ship.cargo.capacity}{" "}
                      {ship.nav.status} {ship.nav.waypointSymbol}
                    </Flex>
                  }
                >
                  <Link to={`/ships/${ship.symbol}`}>
                    {ship.symbol} (
                    {ship.status.status.__typename.replace("Status", "")}) (
                    {ship.status.tempAssignmentId || ship.status.assignmentId})
                    ({ship.status.tempFleetId || ship.status.fleetId})
                  </Link>
                </Popover>
              </List.Item>
            )}
          />
        </Card>
        <Card
          size="small"
          title={`Fleets in System (${system?.fleets?.length || 0} - ${system?.fleets?.flatMap((f) => f.assignments).length || 0})`}
        >
          <List
            size="small"
            style={{ maxHeight: "200px", overflowY: "auto" }}
            dataSource={system?.fleets}
            renderItem={(fleet) => (
              <List.Item>
                <Popover
                  title={<AssignmentsPopover assignments={fleet.assignments} />}
                >
                  <Link to={`/fleets/${fleet.id}`}>
                    {fleet.fleetType}_{fleet.id} ({fleet.active ? "A" : "I"}) (
                    {
                      fleet.assignments.filter((asgmt) => asgmt.ship.length > 0)
                        .length
                    }
                    /{fleet.assignments.length}){" "}
                    {fleet.config.__typename === "TradingConfig"
                      ? `(${fleet.config.tradeMode})`
                      : ""}
                    {fleet.config.__typename === "ChartingConfig"
                      ? `(${fleet.config.chartOnlyJumpGates ? "Gate" : "System"})`
                      : ""}
                  </Link>
                </Popover>
              </List.Item>
            )}
          />
        </Card>
        <Card size="small" title="Gate Connections">
          {system?.jumpGateConnections &&
            system?.jumpGateConnections.length && (
              <List
                size="small"
                style={{ maxHeight: "200px", overflowY: "auto" }}
                dataSource={[
                  ...new Set(system?.jumpGateConnections.map((e) => e.from)),
                ]}
                renderItem={(wp) => (
                  <List.Item>
                    <WaypointLink waypoint={wp}>{wp}</WaypointLink>
                  </List.Item>
                )}
              />
            )}
          <Divider dashed size="small" />
          <List
            size="small"
            style={{ maxHeight: "200px", overflowY: "auto" }}
            dataSource={system?.jumpGateConnections}
            renderItem={(wp) => (
              <List.Item>
                <WaypointLink waypoint={wp.to}>{wp.to}</WaypointLink>
              </List.Item>
            )}
          />
        </Card>
      </Space>
      <Divider />
      <Descriptions
        bordered
        column={5}
        items={[
          {
            label: "Chart Reward",
            children: (
              <span>
                <MoneyDisplay
                  amount={
                    (system?.chartTransactions || [])
                      .map((s) => s.totalPrice)
                      .reduce((r, e) => {
                        return r + e;
                      }, 0) || 0
                  }
                />
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
                  Median <MoneyDisplay amount={Math.ceil(fuelCost.median)} />
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
                  Min <MoneyDisplay amount={Math.ceil(antimatterCost.min)} />
                </Flex>
                <Flex justify="space-between">
                  Max <MoneyDisplay amount={Math.ceil(antimatterCost.max)} />
                </Flex>
                <Flex justify="space-between">
                  Avg <MoneyDisplay amount={Math.ceil(antimatterCost.avg)} />
                </Flex>
                <Flex justify="space-between">
                  Median{" "}
                  <MoneyDisplay amount={Math.ceil(antimatterCost.median)} />
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
                  <span className="text-right">{marketDistripution?.HIGH}</span>
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

      <Divider />
      {(system?.constructionMaterials || []).length > 0 && (
        <>
          <Table
            title={() => "Construction Materials"}
            size="small"
            columns={[
              {
                title: "Waypoint",
                dataIndex: "waypointSymbol",
                key: "waypointSymbol",
                sorter: (a, b) =>
                  a.waypointSymbol.localeCompare(b.waypointSymbol),
              },
              {
                title: "Last Updated",
                dataIndex: "updatedAt",
                key: "updatedAt",
                sorter: (a, b) => a.updatedAt.localeCompare(b.updatedAt),
                render: (updatedAt: string) => (
                  <span>{new Date(updatedAt).toLocaleString()}</span>
                ),
              },
              {
                title: "trade Symbol",
                dataIndex: "tradeSymbol",
                key: "tradeSymbol",
                sorter: (a, b) => a.tradeSymbol.localeCompare(b.tradeSymbol),
                render: (value) => (
                  <Link to={`/supplyChain/${value}`}>{value}</Link>
                ),
              },
              {
                title: "required",
                dataIndex: "required",
                key: "required",
                sorter: (a, b) => a.required - b.required,
                align: "right",
              },
              {
                title: "fulfilled",
                dataIndex: "fulfilled",
                key: "fulfilled",
                sorter: (a, b) => a.fulfilled - b.fulfilled,
                align: "right",
              },
              {
                title: "Transactions",
                key: "purchaseTransactions",
                render: (_, record) => (
                  <span>
                    {(
                      record.marketTransactionSummary.allPurchaseTransactions ||
                      0
                    ).toLocaleString()}
                  </span>
                ),
                sorter: (a, b) =>
                  (a.marketTransactionSummary.allPurchaseTransactions || 0) -
                  (b.marketTransactionSummary.allPurchaseTransactions || 0),
                align: "right",
              },
              {
                title: "Units",
                key: "purchaseUnits",
                render: (_, record) => (
                  <span>
                    {(
                      record.marketTransactionSummary.allPurchaseUnits || 0
                    ).toLocaleString()}
                  </span>
                ),
                sorter: (a, b) =>
                  (a.marketTransactionSummary.allPurchaseUnits || 0) -
                  (b.marketTransactionSummary.allPurchaseUnits || 0),
                align: "right",
              },
              {
                title: "expenses",
                key: "expenses",
                render: (_, record) => (
                  <MoneyDisplay
                    amount={record.marketTransactionSummary.allExpenses || 0}
                  />
                ),
                sorter: (a, b) =>
                  (a.marketTransactionSummary.allExpenses || 0) -
                  (b.marketTransactionSummary.allExpenses || 0),
                align: "right",
              },
              {
                title: "Percent",
                dataIndex: "",
                key: "percent",
                render: (_, record) => (
                  <>
                    <Progress
                      percent={(record.fulfilled / record.required) * 100}
                      size={"small"}
                    />
                  </>
                ),
              },
            ]}
            dataSource={system?.constructionMaterials || []}
            rowKey={(row) => row.waypointSymbol + row.tradeSymbol}
            pagination={false}
          />
          <Divider />
        </>
      )}
      <WaypointTable waypoints={system?.waypoints.items || []} />
      <Divider />
      <Row gutter={10}>
        <Col span={15}>
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
                    (system?.marketTrades.items || []).map(
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
                render: (value: string) => (
                  <Link to={`/supplyChain/${value}`}>{value}</Link>
                ),
                sorter: (a, b) => a.symbol.localeCompare(b.symbol),
                filters: [
                  ...new Set(
                    (system?.marketTrades.items || []).map((t) => t.symbol),
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
                render: (_, record) => record.marketTradeGood?.supply || "N/A",
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
            dataSource={(system?.marketTrades.items || []).filter(
              (t) => !hideFuelInMarketTrades || t.symbol !== "FUEL",
            )}
            rowKey={(row) => row.symbol + row.waypointSymbol + row.type}
            pagination={{
              showTotal: (total, range) =>
                `${range[0]}-${range[1]} of ${total}`,
            }}
          />
        </Col>
        <Col span={9}>
          <Table
            size="small"
            columns={[
              {
                title: "Waypoint",
                dataIndex: "waypointSymbol",
                key: "waypointSymbol",
                render: (symbol: string | undefined) =>
                  symbol ? (
                    <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
                  ) : (
                    "N/A"
                  ),
                sorter: (a, b) =>
                  (a.waypointSymbol || "").localeCompare(
                    b.waypointSymbol || "",
                  ),

                filters: [
                  ...new Set(
                    system?.chartTransactions.map(
                      (t) => t.waypointSymbol || "",
                    ),
                  ),
                ].map((t) => ({
                  text: t,
                  value: t,
                })),
                onFilter: (value, record) => record.waypointSymbol === value,
              },
              {
                title: "Ship",
                dataIndex: "shipSymbol",
                key: "shipSymbol",
                render: (symbol: string) => (
                  <Link to={`/ships/${symbol}`}>{symbol}</Link>
                ),
                sorter: (a, b) =>
                  (a.shipSymbol || "").localeCompare(b.shipSymbol || ""),
                filters: [
                  ...new Set(
                    system?.chartTransactions.map((t) => t.shipSymbol || ""),
                  ),
                ].map((t) => ({
                  text: t,
                  value: t,
                })),
                onFilter: (value, record) => record.shipSymbol === value,
              },
              {
                title: "Total Price",
                dataIndex: "totalPrice",
                key: "totalPrice",
                render: (value) => <MoneyDisplay amount={value} />,
                align: "right",
                sorter: (a, b) => (a.totalPrice ?? 0) - (b.totalPrice ?? 0),
              },
              {
                title: "Timestamp",
                dataIndex: "timestamp",
                key: "timestamp",
                render: (value) => new Date(value).toLocaleString(),
                align: "right",
                sorter: (a, b) =>
                  new Date(a.timestamp ?? 0).getTime() -
                  new Date(b.timestamp ?? 0).getTime(),
                defaultSortOrder: "descend",
              },
            ]}
            dataSource={system?.chartTransactions || []}
            title={() => "Chart Transactions"}
            rowKey={(row) => row.waypointSymbol}
            pagination={{
              showSizeChanger: true,
              pageSizeOptions: ["10", "20", "50", "100", "200", "500", "1000"],
              defaultPageSize: 10,
              showTotal: (total, range) =>
                `${range[0]}-${range[1]} of ${total}`,
            }}
          />
        </Col>
        <Divider />
        <Col span={15}>
          <Table
            size="small"
            columns={[
              {
                title: "Waypoint",
                dataIndex: "waypointSymbol",
                key: "waypointSymbol",
                render: (symbol: string | undefined) =>
                  symbol ? (
                    <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
                  ) : (
                    "N/A"
                  ),
                sorter: (a, b) =>
                  (a.waypointSymbol || "").localeCompare(
                    b.waypointSymbol || "",
                  ),

                filters: [
                  ...new Set(
                    system?.shipyardShips.map((t) => t.waypointSymbol || ""),
                  ),
                ].map((t) => ({
                  text: t,
                  value: t,
                })),
                onFilter: (value, record) => record.waypointSymbol === value,
              },
              {
                title: "Ship Type",
                dataIndex: "shipType",
                key: "shipType",
                render: (shipType, record) => (
                  <Popover
                    title={
                      <Flex vertical>
                        <Flex justify="space-between" gap={10}>
                          <span>Frame:</span> {record.frameType}
                        </Flex>
                        <Flex justify="space-between" gap={10}>
                          <span>Engine:</span> {record.engineType}
                        </Flex>
                        <Flex justify="space-between" gap={10}>
                          <span>Reactor:</span> {record.reactorType}
                        </Flex>
                        <Flex justify="space-between" gap={10}>
                          <span>Mounts:</span> {record.mounts.join(", ")}
                        </Flex>
                        <Flex justify="space-between" gap={10}>
                          <span>Modules:</span> {record.modules.join(", ")}
                        </Flex>
                      </Flex>
                    }
                  >
                    {shipType}
                  </Popover>
                ),
                sorter: (a, b) => a.shipType.localeCompare(b.shipType),
                filters: Object.values(ShipType).map((shipType) => ({
                  text: shipType,
                  value: shipType,
                })),
                onFilter: (value, record) => record.shipType === value,
              },
              {
                title: "Purchase Price",
                dataIndex: "purchasePrice",
                key: "purchasePrice",
                align: "right",
                render: (price: number) => <MoneyDisplay amount={price} />,
                sorter: (a, b) => a.purchasePrice - b.purchasePrice,
              },
              {
                title: "Supply Level",
                dataIndex: "supply",
                key: "supply",
                filters: Object.values(SupplyLevel).map((supply) => ({
                  text: supply,
                  value: supply,
                })),
                onFilter: (value, record) => record.supply === value,
                sorter: (a, b) => a.supply.localeCompare(b.supply),
              },
              {
                title: "Activity",
                dataIndex: "activity",
                key: "activity",
                sorter: (a, b) =>
                  (a.activity ?? "").localeCompare(b.activity ?? ""),
                filters: Object.values(ActivityLevel).map((activity) => ({
                  text: activity,
                  value: activity,
                })),
              },
              {
                title: "Created At",
                dataIndex: "createdAt",
                key: "createdAt",
                render: (date: string) => (
                  <span>{new Date(date).toLocaleString()}</span>
                ),
              },
            ]}
            dataSource={system?.shipyardShips || []}
            rowKey={(row) => row.waypointSymbol + row.shipType}
            title={() => "Shipyard Ships"}
            pagination={{
              showSizeChanger: true,
              pageSizeOptions: ["10", "20", "50", "100", "200", "500", "1000"],
              defaultPageSize: 10,
              showTotal: (total, range) =>
                `${range[0]}-${range[1]} of ${total}`,
            }}
          />
        </Col>
        <Col span={9}>
          <Table
            size="small"
            columns={[
              {
                title: "Waypoint",
                dataIndex: "waypointSymbol",
                key: "waypointSymbol",
                render: (symbol: string | undefined) =>
                  symbol ? (
                    <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
                  ) : (
                    "N/A"
                  ),
                sorter: (a, b) =>
                  (a.waypointSymbol || "").localeCompare(
                    b.waypointSymbol || "",
                  ),

                filters: [
                  ...new Set(
                    system?.shipyardTransactions.map(
                      (t) => t.waypointSymbol || "",
                    ),
                  ),
                ].map((t) => ({
                  text: t,
                  value: t,
                })),
                onFilter: (value, record) => record.waypointSymbol === value,
              },
              {
                title: "Agent",
                dataIndex: "agentSymbol",
                key: "agentSymbol",
                render: (symbol: string) => (
                  <Link to={`/ships/${symbol}`}>{symbol}</Link>
                ),
                sorter: (a, b) =>
                  (a.agentSymbol || "").localeCompare(b.agentSymbol || ""),
                filters: [
                  ...new Set(
                    system?.shipyardTransactions.map(
                      (t) => t.agentSymbol || "",
                    ),
                  ),
                ].map((t) => ({
                  text: t,
                  value: t,
                })),
                onFilter: (value, record) => record.agentSymbol === value,
              },
              {
                title: "Ship Type",
                dataIndex: "shipType",
                key: "shipType",
                sorter: (a, b) =>
                  (a.shipType || "").localeCompare(b.shipType || ""),
                filters: Object.values(ShipType)
                  .sort((a, b) => a.localeCompare(b))
                  .map((type) => ({
                    text: type,
                    value: type,
                  })),
                onFilter: (value, record) => record.shipType === value,
              },
              {
                title: "Price",
                dataIndex: "price",
                key: "price",
                render: (value) => <MoneyDisplay amount={value} />,
                align: "right",
                sorter: (a, b) => (a.price ?? 0) - (b.price ?? 0),
              },
              {
                title: "Timestamp",
                dataIndex: "timestamp",
                key: "timestamp",
                render: (value) => new Date(value).toLocaleString(),
                align: "right",
                sorter: (a, b) =>
                  new Date(a.timestamp ?? 0).getTime() -
                  new Date(b.timestamp ?? 0).getTime(),
                defaultSortOrder: "descend",
              },
            ]}
            dataSource={system?.shipyardTransactions || []}
            rowKey={(row) => row.id}
            title={() => "Shipyard Transactions"}
            pagination={{
              showSizeChanger: true,
              pageSizeOptions: ["10", "20", "50", "100", "200", "500", "1000"],
              defaultPageSize: 10,
              showTotal: (total, range) =>
                `${range[0]}-${range[1]} of ${total}`,
            }}
          />
        </Col>
      </Row>
      <Divider />
      <Row gutter={10}>
        <Col span={14}>
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
                render: (value) => (
                  <Link to={`/supplyChain/${value}`}>{value}</Link>
                ),
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
                      amount={record.marketTransactionSummary?.allExpenses || 0}
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
                              (record.purchaseMarketTradeGood?.purchasePrice ||
                                0) * record.tradeVolume || 0
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
                              ((record.purchaseMarketTradeGood?.purchasePrice ||
                                0) * record.tradeVolume || 0) -
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
                              record.marketTransactionSummary?.allExpenses || 0
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
                          (record.marketTransactionSummary?.allExpenses || 0) >
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
            dataSource={system?.tradeRoutes || []}
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
        <Col span={10}>
          <Table
            size="small"
            columns={[
              {
                title: "Contract",
                dataIndex: "contractId",
                key: "contractId",
                render: (id: string) => (
                  <Link to={`/contracts/${id}`}>
                    {id.slice(0, 3)}...{id.slice(-3)}
                  </Link>
                ),
                sorter: (a, b) => a.contractId.localeCompare(b.contractId),
              },
              {
                title: "Trade Symbol",
                dataIndex: "tradeSymbol",
                key: "tradeSymbol",
                sorter: (a, b) => a.tradeSymbol.localeCompare(b.tradeSymbol),
                filters: Object.values(TradeSymbol).map((sym) => ({
                  text: sym,
                  value: sym,
                })),
                onFilter: (value, record) => record.tradeSymbol === value,
                render: (value) => (
                  <Link to={`/supplyChain/${value}`}>{value}</Link>
                ),
              },
              {
                title: "Destination",
                dataIndex: "destinationSymbol",
                key: "destinationSymbol",
                render: (symbol: string) => (
                  <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
                ),
                sorter: (a, b) =>
                  a.destinationSymbol.localeCompare(b.destinationSymbol),
              },
              {
                title: "Required",
                dataIndex: "unitsRequired",
                key: "unitsRequired",
                align: "right",
                sorter: (a, b) => a.unitsRequired - b.unitsRequired,
              },
              {
                title: "Fulfilled",
                dataIndex: "unitsFulfilled",
                key: "unitsFulfilled",
                align: "right",
                sorter: (a, b) => a.unitsFulfilled - b.unitsFulfilled,
              },
              {
                title: "Faction",
                key: "faction",
                render: (_, record) => record.contract?.factionSymbol || "N/A",
                sorter: (a, b) =>
                  (a.contract?.factionSymbol || "").localeCompare(
                    b.contract?.factionSymbol || "",
                  ),
              },
              {
                title: "Profit",
                key: "profit",
                align: "right",
                render: (_, record) => (
                  <Popover
                    content={
                      <Flex vertical>
                        <Flex justify="space-between" gap={10}>
                          <span>On Fulfilled:</span>{" "}
                          <MoneyDisplay
                            amount={record.contract?.onFulfilled || 0}
                          />
                        </Flex>
                        <Flex justify="space-between" gap={10}>
                          <span>On Accepted:</span>{" "}
                          <MoneyDisplay
                            amount={record.contract?.onAccepted || 0}
                          />
                        </Flex>
                        <Flex
                          justify="space-between"
                          className="border-t border-gray-600"
                          gap={10}
                        >
                          <span>Total Rewards:</span>{" "}
                          <MoneyDisplay
                            amount={
                              (record.contract?.onAccepted || 0) +
                              (record.contract?.onFulfilled || 0)
                            }
                          />
                        </Flex>
                        <Flex justify="space-between" gap={10}>
                          <span>Expenses:</span>{" "}
                          <MoneyDisplay
                            amount={
                              record.contract?.marketTransactionSummary
                                .allExpenses || 0
                            }
                          />
                        </Flex>
                        <Flex
                          justify="space-between"
                          className="border-t border-gray-600"
                          gap={10}
                        >
                          <span>Profit:</span>{" "}
                          <MoneyDisplay
                            amount={
                              (record.contract?.onFulfilled || 0) +
                              (record.contract?.onAccepted || 0) -
                              (record.contract?.marketTransactionSummary
                                .allExpenses || 0)
                            }
                          />
                        </Flex>
                      </Flex>
                    }
                  >
                    <MoneyDisplay
                      amount={
                        (record.contract?.onFulfilled || 0) +
                        (record.contract?.onAccepted || 0) -
                        (record.contract?.marketTransactionSummary
                          .allExpenses || 0)
                      }
                      className={
                        (record.contract?.onFulfilled || 0) +
                          (record.contract?.onAccepted || 0) -
                          (record.contract?.marketTransactionSummary
                            .allExpenses || 0) >
                        0
                          ? "text-current"
                          : "text-red-600"
                      }
                    />
                  </Popover>
                ),
                sorter: (a, b) =>
                  (a.contract?.onFulfilled || 0) +
                  (a.contract?.onAccepted || 0) -
                  (a.contract?.marketTransactionSummary.allExpenses || 0) -
                  (b.contract?.onFulfilled || 0) -
                  (b.contract?.onAccepted || 0) +
                  (b.contract?.marketTransactionSummary.allExpenses || 0),
              },
              {
                title: "Deadline",
                key: "deadline",
                render: (_, record) =>
                  record.contract?.deadline
                    ? new Date(record.contract.deadline).toLocaleString()
                    : "N/A",
                sorter: (a, b) =>
                  new Date(a.contract?.deadline ?? 0).getTime() -
                  new Date(b.contract?.deadline ?? 0).getTime(),
                defaultSortOrder: "descend",
              },
            ]}
            dataSource={system?.contractDeliveries || []}
            rowKey={(row) =>
              row.contractId + row.tradeSymbol + row.destinationSymbol
            }
            title={() => "Contract Deliveries"}
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
    </div>
  );
}

export default System;
