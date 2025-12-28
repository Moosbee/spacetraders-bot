import {
  DownloadOutlined,
  NodeIndexOutlined,
  SortDescendingOutlined,
  TruckOutlined,
  UploadOutlined,
} from "@ant-design/icons";
import { useQuery } from "@apollo/client/react";
import {
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
  Spin,
  Table,
  TableProps,
} from "antd";
import { Link, useParams } from "react-router-dom";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import Timer from "../features/Timer/Timer";
import TransactionTable from "../features/TransactionTable/TransactionTable";
import WaypointLink from "../features/WaypointLink";
import {
  ActivityLevel,
  GetSystemQuery,
  ShipType,
  SupplyLevel,
  TradeSymbol,
  WaypointModifierSymbol,
  WaypointTraitSymbol,
  WaypointType,
} from "../gql/graphql";
import { GET_SYSTEM } from "../graphql/queries";
import { useAppDispatch, useAppSelector } from "../redux/hooks";
import {
  selectSelectedSystemSymbol,
  setSelectedSystemSymbol,
} from "../redux/slices/mapSlice";
import { systemIcons } from "../utils/waypointColors";

function System() {
  const { systemID } = useParams();

  const { loading, error, data, dataState, refetch } = useQuery(GET_SYSTEM, {
    variables: { systemSymbol: systemID || "" },
  });

  const selectedSystem = useAppSelector(selectSelectedSystemSymbol);

  const dispatch = useAppDispatch();

  // if (dataState != "complete") return <p>Loading...</p>;
  if (error) return <p>Error: {error.message}</p>;

  const system = data?.system;

  type GQLWaypoint = GetSystemQuery["system"]["waypoints"][number];

  const color = systemIcons[system?.systemType || "BLACK_HOLE"].color;
  const waypointIcon = systemIcons[system?.systemType || "BLACK_HOLE"].icon;

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
            {waypointIcon}
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
          <Spin spinning={loading || dataState !== "complete"} />
          <Button
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
      children: system?.fleets.length,
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
      children: `${system?.waypoints.filter((wp) => wp.chartedOn).length}/${
        system?.waypoints.length
      }`,
    },
    {
      label: "Marketplaces",
      key: "marketplaces",
      children: `${
        system?.waypoints
          .filter((wp) => wp.hasMarketplace)
          .filter((wp) => wp.chartedOn).length
      }/${system?.waypoints.filter((wp) => wp.hasMarketplace).length}`,
    },
    {
      label: "Shipyards",
      key: "shipyards",
      children: `${
        system?.waypoints
          .filter((wp) => wp.hasShipyard)
          .filter((wp) => wp.chartedOn).length
      }/${system?.waypoints.filter((wp) => wp.hasShipyard).length}`,
    },
  ];

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
                {symbol.replace(system?.symbol + "-", "")}
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
            {orbits.replace(system?.symbol + "-", "")}
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
                    60
                )}
                min{" "}
                {Math.floor(
                  ((new Date(record.nextScrap).getTime() -
                    new Date(record.lastScrap).getTime()) /
                    1000) %
                    60
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
          b.nextScrap ?? (sortOrder == "ascend" ? "9" : "0")
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
        marketTrades && marketTrades.length > 0 ? (
          <>
            {/* <Flex gap={1} vertical>
              {marketTrades.map((trade_good) => (
                <span>
                  {trade_good.type.slice(0, 3)} {trade_good.symbol}
                </span>
              ))}
            </Flex> */}
            <Popover
              content={
                <Flex gap={1} vertical>
                  {marketTrades.filter((t) => t.type === "EXCHANGE").length >
                    0 && <span className="font-bold">EXCHANGE</span>}
                  {marketTrades
                    .filter((t) => t.type === "EXCHANGE")
                    .map((trade_good) => (
                      <Flex justify="space-between" key={trade_good.symbol}>
                        <span>{trade_good.symbol}</span>
                        <Flex gap={1} justify="end">
                          <span className="text-nowrap">
                            <UploadOutlined />{" "}
                            <MoneyDisplay
                              amount={
                                trade_good.marketTradeGood?.purchasePrice || 0
                              }
                            />
                          </span>
                          |
                          <span className="text-nowrap">
                            <DownloadOutlined />{" "}
                            <MoneyDisplay
                              amount={
                                trade_good.marketTradeGood?.sellPrice || 0
                              }
                            />
                          </span>
                          |
                          <span className="text-nowrap">
                            <TruckOutlined />{" "}
                            {trade_good.marketTradeGood?.tradeVolume}
                          </span>
                          |
                          <span>
                            {trade_good.marketTradeGood?.supply.slice(0, 3)}
                          </span>
                        </Flex>
                      </Flex>
                    ))}
                  {marketTrades.filter((t) => t.type === "IMPORT").length >
                    0 && <span className="font-bold">IMPORT</span>}

                  {marketTrades
                    .filter((t) => t.type === "IMPORT")
                    .map((trade_good) => (
                      <Flex justify="space-between" key={trade_good.symbol}>
                        <span>{trade_good.symbol}</span>
                        <Flex gap={1} justify="end">
                          <span className="text-nowrap">
                            <UploadOutlined />{" "}
                            <MoneyDisplay
                              amount={
                                trade_good.marketTradeGood?.purchasePrice || 0
                              }
                            />
                          </span>
                          |
                          <span className="font-bold text-nowrap">
                            <DownloadOutlined />{" "}
                            <MoneyDisplay
                              amount={
                                trade_good.marketTradeGood?.sellPrice || 0
                              }
                            />
                          </span>
                          |
                          <span className="text-nowrap">
                            <TruckOutlined />{" "}
                            {trade_good.marketTradeGood?.tradeVolume}
                          </span>
                          |
                          <span>
                            {trade_good.marketTradeGood?.supply.slice(0, 3)}
                          </span>
                        </Flex>
                      </Flex>
                    ))}
                  {marketTrades.filter((t) => t.type === "EXPORT").length >
                    0 && <span className="font-bold">EXPORT</span>}

                  {marketTrades
                    .filter((t) => t.type === "EXPORT")
                    .map((trade_good) => (
                      <Flex justify="space-between" key={trade_good.symbol}>
                        <span>{trade_good.symbol}</span>
                        <Flex gap={1} justify="end">
                          <span className="text-nowrap font-bold">
                            <UploadOutlined />{" "}
                            <MoneyDisplay
                              amount={
                                trade_good.marketTradeGood?.purchasePrice || 0
                              }
                            />
                          </span>
                          |
                          <span className="text-nowrap">
                            <DownloadOutlined />{" "}
                            <MoneyDisplay
                              amount={
                                trade_good.marketTradeGood?.sellPrice || 0
                              }
                            />
                          </span>
                          |
                          <span className="text-nowrap">
                            <TruckOutlined />{" "}
                            {trade_good.marketTradeGood?.tradeVolume}
                          </span>
                          |
                          <span>
                            {trade_good.marketTradeGood?.supply.slice(0, 3)}
                          </span>
                        </Flex>
                      </Flex>
                    ))}
                  {marketTrades.filter((t) => t.type === "EXPORT").length >
                    0 && <span className="font-bold">MAPPING</span>}
                  <div className="flex flex-col">
                    {marketTrades
                      .filter((t) => t.type === "EXPORT")
                      .map((trade_good) => (
                        <div
                          key={trade_good.symbol + "EXPORT" + trade_good.type}
                          className={`flex justify-between border-t-2 border-t-current`}
                        >
                          <div className="flex flex-col">
                            {trade_good.tradeSymbolInfo.requires.map((t) => (
                              <div
                                className={`${
                                  marketTrades.some(
                                    (e) =>
                                      e.type === "IMPORT" &&
                                      e.symbol == t.symbol
                                  )
                                    ? "text-current"
                                    : "text-red-700"
                                }`}
                              >
                                {t.symbol}
                              </div>
                            ))}
                          </div>
                          <div className="flex items-center">
                            {trade_good.symbol}
                          </div>
                        </div>
                      ))}
                  </div>
                </Flex>
              }
            >
              <Flex gap={1} flex={1} vertical>
                {marketTrades.filter((t) => t.type === "EXCHANGE").length >
                  0 && (
                  <Flex justify="space-between">
                    <span>EXCHANGE</span>
                    <span>
                      {marketTrades.filter((t) => t.type === "EXCHANGE").length}
                    </span>
                  </Flex>
                )}
                {marketTrades.filter((t) => t.type === "IMPORT").length > 0 && (
                  <Flex justify="space-between">
                    <span>IMPORT</span>
                    <span>
                      {marketTrades.filter((t) => t.type === "IMPORT").length}
                    </span>
                  </Flex>
                )}
                {marketTrades.filter((t) => t.type === "EXPORT").length > 0 && (
                  <Flex justify="space-between">
                    <span>EXPORT</span>
                    <span>
                      {marketTrades.filter((t) => t.type === "EXPORT").length}
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
        (a.marketTrades?.length ?? 0) - (b.marketTrades?.length ?? 0),
      filters: Object.values(TradeSymbol).map((trade_good) => ({
        text: trade_good,
        value: trade_good,
      })),
      onFilter: (value, record) =>
        record.marketTrades?.some((t) => t.symbol === value) ?? false,
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
              {value.map((ship) => (
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
          Ships {value.length}
        </Popover>
      ),
      sorter: (a, b) =>
        (a.shipyardShips.length ?? 0) - (b.shipyardShips.length ?? 0),
      filters: [
        ...[...new Set(system?.shipyardShips.map((ship) => ship.shipType))].map(
          (sh) => ({
            text: sh,
            value: sh,
          })
        ),
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
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title={`System ${systemID}`} />
      <h2>System {systemID}</h2>
      <Space>
        <Descriptions bordered column={3} items={items} />

        <Card size="small" title="Known Agents">
          <List
            size="small"
            dataSource={[...(system?.seenAgents || [])].sort(
              (a, b) => b.count - a.count
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

        <Card size="small" title="Ships in System">
          <List
            size="small"
            style={{ maxHeight: "200px", overflowY: "auto" }}
            dataSource={system?.ships}
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
        <Card size="small" title="Fleets in System">
          <List
            size="small"
            style={{ maxHeight: "200px", overflowY: "auto" }}
            dataSource={system?.fleets}
            renderItem={(fleet) => (
              <List.Item>
                <Popover
                  title={
                    <Flex flex={1} vertical>
                      {fleet.assignments.map((asgmt) => (
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
                  <Link to={`/fleets/${fleet.id}`}>
                    {fleet.fleetType}_{fleet.id} ({fleet.active ? "A" : "I"}) (
                    {fleet.assignments.length})
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
                    system?.chartTransactions
                      .map((s) => s.totalPrice)
                      .reduce((r, e) => {
                        return r + e;
                      }) || 0
                  }
                />
              </span>
            ),
          },
        ]}
      />

      <Divider />
      {(system?.constructionMaterials || []).length > 0 && (
        <>
          <Table
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
                title: "trade Symbol",
                dataIndex: "tradeSymbol",
                key: "tradeSymbol",
                sorter: (a, b) => a.tradeSymbol.localeCompare(b.tradeSymbol),
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
                      record.marketTransactionSummary.purchaseTransactions || 0
                    ).toLocaleString()}
                  </span>
                ),
                sorter: (a, b) =>
                  (a.marketTransactionSummary.purchaseTransactions || 0) -
                  (b.marketTransactionSummary.purchaseTransactions || 0),
                align: "right",
              },
              {
                title: "Units",
                key: "purchaseUnits",
                render: (_, record) => (
                  <span>
                    {(
                      record.marketTransactionSummary.purchaseUnits || 0
                    ).toLocaleString()}
                  </span>
                ),
                sorter: (a, b) =>
                  (a.marketTransactionSummary.purchaseUnits || 0) -
                  (b.marketTransactionSummary.purchaseUnits || 0),
                align: "right",
              },
              {
                title: "expenses",
                key: "expenses",
                render: (_, record) => (
                  <MoneyDisplay
                    amount={record.marketTransactionSummary.expenses || 0}
                  />
                ),
                sorter: (a, b) =>
                  (a.marketTransactionSummary.expenses || 0) -
                  (b.marketTransactionSummary.expenses || 0),
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
      <Table
        size="middle"
        columns={columns}
        dataSource={system?.waypoints || []}
        rowKey={(row) => row.symbol}
        pagination={{
          showTotal: (total, range) => `${range[0]}-${range[1]} of ${total}`,
        }}
      />
      <Divider />
      <Row gutter={10}>
        <Col span={15}>
          <TransactionTable
            transactions={system?.marketTransactions || []}
            reasons={{
              contract: false,
              construction_shipment_id: false,
              mining: false,
              trade_route_id: false,
            }}
            size="small"
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
                    b.waypointSymbol || ""
                  ),

                filters: [
                  ...new Set(
                    system?.chartTransactions.map((t) => t.waypointSymbol || "")
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
                    system?.chartTransactions.map((t) => t.shipSymbol || "")
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
                    b.waypointSymbol || ""
                  ),

                filters: [
                  ...new Set(
                    system?.shipyardShips.map((t) => t.waypointSymbol || "")
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
                    b.waypointSymbol || ""
                  ),

                filters: [
                  ...new Set(
                    system?.shipyardTransactions.map(
                      (t) => t.waypointSymbol || ""
                    )
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
                    system?.shipyardTransactions.map((t) => t.agentSymbol || "")
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
    </div>
  );
}

export default System;
