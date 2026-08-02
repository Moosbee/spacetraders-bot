import {
  Button,
  Descriptions,
  DescriptionsProps,
  Divider,
  Flex,
  List,
  Progress,
  Result,
  Space,
  Table,
  TableProps,
} from "antd";
import { useMemo } from "react";
import { Link, useParams } from "react-router-dom";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import Timer from "../features/Timer/Timer";
import WaypointLink from "../features/WaypointLink";

import { useMutation, useQuery } from "@apollo/client/react";
import { backendUrl } from "../data";
import ShipyardShipTable from "../features/ShipyardShipTable/ShipyardShipTable";
import TransactionTable from "../features/TransactionTable/TransactionTable";
import {
  ActivityLevel,
  GetWaypointQuery,
  SupplyLevel,
  TradeSymbol,
  Type,
} from "../gql/graphql";
import { REPOPULATE_SYSTEMS_WITH_FLEETS_FROM_JUMP_GATE } from "../graphql/mutations";
import { GET_WAYPOINT } from "../graphql/queries";
import { ShipType } from "../models/api";
import { WaypointResponse } from "../models/SQLWaypoint";
import { useAppDispatch, useAppSelector } from "../redux/hooks";
import {
  selectSelectedWaypointSymbol,
  setSelectedWaypointSymbol,
} from "../redux/slices/mapSlice";
import { selectAllShipsArray } from "../redux/slices/shipSlice";
import { message } from "../utils/antdMessage";
import { cn } from "../utils/utils";
import { waypointIcons } from "../utils/waypointColors";

type DataWaypoint = GetWaypointQuery["waypoint"];

function Waypoint() {
  const { systemID } = useParams();
  const { waypointID } = useParams();

  const dispatch = useAppDispatch();

  const selectedWaypoint = useAppSelector(selectSelectedWaypointSymbol);

  const { loading, error, data, refetch } = useQuery(GET_WAYPOINT, {
    variables: { waypointSymbol: waypointID || "" },
  });

  const [
    repopulateSystemsFromJumpGates,
    {
      loading: repopulateSystemsFromJumpGatesLoading,
      error: repopulateSystemsFromJumpGatesError,
    },
  ] = useMutation(REPOPULATE_SYSTEMS_WITH_FLEETS_FROM_JUMP_GATE);

  const oldWaypoint: WaypointResponse | null =
    null as unknown as WaypointResponse;

  const waypoint = data?.waypoint;

  const ships = useAppSelector(selectAllShipsArray);

  const onSystemsShips = useMemo(() => {
    return ships.filter((ship) => ship.nav.waypoint_symbol === waypointID);
  }, [waypointID, ships]);

  if (error) {
    return (
      <Result
        status="error"
        title="Waypoint Error"
        subTitle={`Error: ${error.message}`}
        extra={[
          <Button key="tryAgain" type="primary" onClick={() => refetch()}>
            Try Again
          </Button>,
        ]}
      ></Result>
    );
  }

  const color = waypointIcons[waypoint?.waypointType || "NEBULA"].color;
  const waypointIcon = waypointIcons[waypoint?.waypointType || "NEBULA"].icon;

  const items: DescriptionsProps["items"] = [
    {
      label: "Symbol",
      key: "symbol",
      children: (
        <button
          onClick={() => {
            if (selectedWaypoint?.waypointSymbol === waypoint?.symbol) {
              dispatch(setSelectedWaypointSymbol(undefined));
              return;
            }
            dispatch(
              setSelectedWaypointSymbol(
                waypoint
                  ? {
                      systemSymbol: waypoint.systemSymbol,
                      waypointSymbol: waypoint.symbol,
                    }
                  : undefined,
              ),
            );
          }}
          className="cursor-pointer flex justify-between w-full py-2 flex-nowrap text-nowrap"
        >
          <div
            style={{
              color: color,
            }}
            className="h-6 w-6 flex justify-center items-center text-xl mr-2"
          >
            <span
              className="absolute"
              style={{
                boxShadow:
                  selectedWaypoint?.waypointSymbol == waypoint?.symbol
                    ? "0px 0px calc(0.8 * 1.25rem) calc(0.6 * 1.25rem) color-mix(in srgb, currentColor 80%, #fff 20%)"
                    : "",
              }}
            ></span>
            {waypointIcon}
          </div>
          {waypoint?.symbol}
        </button>
      ),
    },
    {
      label: "System Symbol",
      key: "systemSymbol",
      children: <Link to={`/system/${systemID}`}>{systemID}</Link>,
    },
    {
      label: "Waypoint Type",
      key: "waypointType",
      children: waypoint?.waypointType,
    },
    {
      label: "Coordinates",
      key: "coordinates",
      children: (
        <span>
          <span className="text-nowrap">X: {waypoint?.x}</span>{" "}
          <span className="text-nowrap">Y: {waypoint?.y}</span>
        </span>
      ),
    },
    {
      key: "chart",
      label: "Chart",
      children: (
        <p>
          By: {waypoint?.chartedBy} <br />
          On:{" "}
          {new Date(
            waypoint?.chartedOn ? waypoint.chartedOn : 0,
          ).toLocaleDateString()}
        </p>
      ),
    },
    {
      label: "Faction",
      key: "faction",
      children: waypoint?.faction || "None",
    },

    {
      label: "Orbits",
      key: "orbits",
      children: waypoint?.orbits || "None",
    },
    {
      label: "Orbitals",
      key: "orbitals",
      children:
        (waypoint?.orbitals?.length || 0) > 0 ? (
          <List
            size="small"
            dataSource={waypoint?.orbitals?.map((orbitals) => (
              <WaypointLink waypoint={orbitals}>{orbitals}</WaypointLink>
            ))}
            renderItem={(item) => <List.Item>{item}</List.Item>}
          ></List>
        ) : (
          "None"
        ),
    },
    {
      label: "Modifiers",
      key: "modifiers",
      children:
        (waypoint?.modifiers?.length || 0) > 0 ? (
          <List
            size="small"
            dataSource={waypoint?.modifiers?.map((modifier) => (
              <span>{modifier}</span>
            ))}
            renderItem={(item) => <List.Item>{item}</List.Item>}
          ></List>
        ) : (
          "None"
        ),
    },
    {
      key: "traits",
      label: "Traits",
      span:
        3 -
        (waypoint?.isUnderConstruction ? 1 : 0) -
        (waypoint?.unstableSince ? 1 : 0),
      children: (
        <List
          size="small"
          dataSource={waypoint?.traits.map((trait) => (
            <span>{trait}</span>
          ))}
          renderItem={(item) => <List.Item>{item}</List.Item>}
        ></List>
      ),
    },
    {
      key: "has_shipyard",
      label: "Has Shipyard",
      children: <p>{waypoint?.hasShipyard ? "Yes" : "No"}</p>,
    },
    {
      key: "has_marketplace",
      label: "Has Marketplace",
      children: <p>{waypoint?.hasMarketplace ? "Yes" : "No"}</p>,
    },
  ];

  if (waypoint?.isUnderConstruction) {
    items.push({
      key: "is_under_construction",
      label: "Under Construction",
      children: <p>{waypoint?.isUnderConstruction ? "Yes" : "No"}</p>,
    });
  }

  if (waypoint?.unstableSince) {
    items.push({
      key: "unstable_since",
      label: "Unstable Since",
      children: (
        <p>
          <Timer time={waypoint.unstableSince} />
        </p>
      ),
    });
  }

  const marketTradeColumns: TableProps<
    DataWaypoint["marketTrades"]["items"][number]
  >["columns"] = [
    {
      title: "Symbol",
      dataIndex: "symbol",
      key: "symbol",
      sorter: (a, b) => a.symbol.localeCompare(b.symbol),
      filters: [
        ...new Set((waypoint?.marketTrades.items || []).map((t) => t.symbol)),
      ].map((t) => ({
        text: t,
        value: t,
      })),
      onFilter: (value, record) => record.symbol === value,
    },
    {
      title: "Type",
      dataIndex: "type",
      key: "type",
      sorter: (a, b) => a.type.localeCompare(b.type),
      filters: Object.values(Type).map((t) => ({
        text: t,
        value: t,
      })),
      onFilter: (value, record) => record.type === value,
    },
  ];

  const marketTradeGoodsColumns: TableProps<
    DataWaypoint["marketTradeGoods"]["items"][number]
  >["columns"] = [
    {
      title: "Symbol",
      dataIndex: "symbol",
      key: "symbol",
      sorter: (a, b) => a.symbol.localeCompare(b.symbol),
      filters: [
        ...new Set(
          (waypoint?.marketTradeGoods.items || []).map((t) => t.symbol),
        ),
      ].map((t) => ({
        text: t,
        value: t,
      })),
      onFilter: (value, record) => record.symbol === value,
    },
    {
      title: "Type",
      dataIndex: "type",
      key: "type",
      sorter: (a, b) => a.type.localeCompare(b.type),
      filters: Object.values(Type).map((t) => ({
        text: t,
        value: t,
      })),
      onFilter: (value, record) => record.type === value,
    },

    {
      title: "Supply",
      dataIndex: "supply",
      key: "supply",
      sorter: (a, b) => a.supply.localeCompare(b.supply),
      filters: Object.values(SupplyLevel).map((t) => ({
        text: t,
        value: t,
      })),
      onFilter: (value, record) => record.supply === value,
    },
    {
      title: "Volume",
      dataIndex: "tradeVolume",
      key: "tradeVolume",
      align: "right",

      sorter: (a, b) => a.tradeVolume - b.tradeVolume,
    },
    {
      title: "Activity",
      dataIndex: "activity",
      key: "activity",
      render: (activity) => activity || "N/A",
      sorter: (a, b) => (a.activity ?? "").localeCompare(b.activity ?? ""),
      filters: Object.values(ActivityLevel).map((t) => ({
        text: t,
        value: t,
      })),
      onFilter: (value, record) => record.activity === value,
    },
    {
      title: "Purchase",
      dataIndex: "purchasePrice",
      key: "purchasePrice",
      render: (purchasePrice, data) =>
        data.type === "IMPORT" ? (
          <MoneyDisplay amount={purchasePrice} />
        ) : (
          <b>
            <MoneyDisplay amount={purchasePrice} />
          </b>
        ),
      sorter: (a, b) => a.purchasePrice - b.purchasePrice,
    },
    {
      title: "Sell",
      dataIndex: "sellPrice",
      key: "sellPrice",
      render: (sellPrice, data) =>
        data.type === "EXPORT" ? (
          <MoneyDisplay amount={sellPrice} />
        ) : (
          <b>
            <MoneyDisplay amount={sellPrice} />
          </b>
        ),
      sorter: (a, b) => a.sellPrice - b.sellPrice,
    },
    {
      title: "Created At",
      dataIndex: "createdAt",
      key: "createdAt",
      render: (date: string) => new Date(date).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime(),
    },
  ];

  const shipTypesColumns: TableProps<
    DataWaypoint["shipyardShips"]["items"][number]
  >["columns"] = [
    {
      title: "Ship Type",
      dataIndex: "shipType",
      key: "shipType",
      sorter: (a, b) => a.shipType.localeCompare(b.shipType),
      filters: Object.values(ShipType).map((t) => ({
        text: t,
        value: t,
      })),
      onFilter: (value, record) => record.shipType === value,
    },
    {
      title: "Created At",
      dataIndex: "createdAt",
      key: "createdAt",
      render: (date: string) => new Date(date).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime(),
    },
  ];

  const shipTransactionColumns: TableProps<
    DataWaypoint["shipyardTransactions"]["items"][number]
  >["columns"] = [
    {
      title: "Agent Symbol",
      dataIndex: "agentSymbol",
      key: "agentSymbol",
      sorter: (a, b) => a.agentSymbol.localeCompare(b.agentSymbol),
    },
    {
      title: "Ship Type",
      dataIndex: "shipType",
      key: "shipType",
      sorter: (a, b) => a.shipType.localeCompare(b.shipType),
      filters: Object.values(ShipType).map((t) => ({
        text: t,
        value: t,
      })),
      onFilter: (value, record) => record.shipType === value,
    },
    {
      title: "Price",
      dataIndex: "price",
      key: "price",
      align: "right",
      sorter: (a, b) => a.price - b.price,
      render: (price: number) => <MoneyDisplay amount={price} />,
    },
    {
      title: "Timestamp",
      dataIndex: "timestamp",
      key: "timestamp",
      render: (date: string) => new Date(date).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime(),
      defaultSortOrder: "descend",
    },
  ];

  const constructionMaterialColumns: TableProps<
    DataWaypoint["constructionMaterials"]["items"][number]
  >["columns"] = [
    {
      title: "Trade Symbol",
      dataIndex: "tradeSymbol",
      key: "tradeSymbol",
      sorter: (a, b) => a.tradeSymbol.localeCompare(b.tradeSymbol),
    },
    {
      title: "Required",
      dataIndex: "required",
      key: "required",
      align: "right",
      sorter: (a, b) => a.required - b.required,
    },
    {
      title: "Fulfilled",
      dataIndex: "fulfilled",
      key: "fulfilled",
      align: "right",
      sorter: (a, b) => a.fulfilled - b.fulfilled,
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
        <MoneyDisplay amount={record.marketTransactionSummary.expenses || 0} />
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
    {
      title: "Last Updated",
      dataIndex: "updatedAt",
      key: "updatedAt",
      render: (date: string) => new Date(date).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.updatedAt).getTime() - new Date(b.updatedAt).getTime(),
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title={`Waypoint ${waypointID}`} />
      <Space>
        <h2 className="text-2xl font-bold">
          Waypoint {waypointID} in {systemID}
        </h2>
        <Button
          onClick={() => {
            refetch();
          }}
          loading={loading}
        >
          Reload
        </Button>
        {waypoint?.hasShipyard && <a href="#shipyard">Shipyard</a>}
        {waypoint?.hasMarketplace && (
          <Link to={`/system/${systemID}/${waypointID}/marketHistory`}>
            Market History
          </Link>
        )}
        {onSystemsShips.map((s) => (
          <Link to={`/ships/${s.symbol}`}>{s.symbol}</Link>
        ))}
      </Space>
      <Divider size="small" />
      <Flex align="stretch" justify="flex-start" gap={24}>
        <Descriptions
          bordered
          column={6}
          items={items}
          layout="vertical"
          size="small"
        />
        <Flex vertical gap={24}>
          {waypoint?.marketTrades.items &&
            waypoint?.marketTrades.items.length > 0 && (
              <Table
                columns={marketTradeColumns}
                dataSource={waypoint?.marketTrades.items}
                rowKey={(symbol) => symbol.symbol}
                size="small"
              />
            )}
          {(waypoint?.marketTrades.items || []).filter(
            (t) => t.type === "EXPORT",
          ).length > 0 && (
            <Table
              size="small"
              dataSource={waypoint?.marketTrades.items
                .filter((t) => t.type === "EXPORT")
                .map((trade_good) => ({
                  export: trade_good,
                  required: trade_good.tradeSymbolInfo.requires.items.map(
                    (t) => ({
                      ...t,
                      fulfilled: waypoint?.marketTrades.items.some(
                        (e) => e.type === "IMPORT" && e.symbol == t.symbol,
                      ),
                    }),
                  ),
                }))}
              columns={[
                {
                  title: "Requires",
                  dataIndex: "required",
                  key: "required",
                  render: (
                    tradeGoods: { symbol: TradeSymbol; fulfilled: boolean }[],
                  ) => (
                    <div className="flex flex-col">
                      {tradeGoods.map((tradeGood) => (
                        <span
                          className={cn(
                            // "not-last:border-b border-gray-600",
                            tradeGood.fulfilled ? "" : "text-red-700",
                          )}
                          key={tradeGood.symbol}
                        >
                          {tradeGood.symbol}
                        </span>
                      ))}
                    </div>
                  ),
                },
                {
                  title: "Export",
                  dataIndex: "export",
                  key: "export",
                  render: (tradeGood) => tradeGood.symbol,
                },
              ]}
            />
          )}
          {waypoint?.jumpGateConnections.items &&
            waypoint.jumpGateConnections.items.length > 0 && (
              <Flex vertical gap={12}>
                <Button
                  loading={repopulateSystemsFromJumpGatesLoading}
                  onClick={() => {
                    repopulateSystemsFromJumpGates({
                      variables: { jumpGate: waypointID || "" },
                    }).then((data) => {
                      message.success(
                        `Repopulated systems with fleets from jump gate ${waypointID} ${data.data?.repopulateSystemsWithFleetsFromJumpGate ? "success" : "failed"}`,
                      );
                    });
                  }}
                >
                  Populate
                </Button>
                {repopulateSystemsFromJumpGatesError && (
                  <p style={{ color: "red" }}>
                    Error: {repopulateSystemsFromJumpGatesError.message}
                  </p>
                )}
                <Table
                  columns={[
                    {
                      title: "Symbol",
                      dataIndex: "to",
                      key: "to",
                      sorter: (a, b) => a.to.localeCompare(b.to),
                      render: (symbol: string) => (
                        <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
                      ),
                    },
                  ]}
                  dataSource={waypoint?.jumpGateConnections.items}
                  rowKey={(symbol) => symbol.id}
                  size="small"
                />
              </Flex>
            )}
        </Flex>
        <Flex vertical gap={24}>
          {waypoint?.marketTradeGoods.items &&
            waypoint.marketTradeGoods.items.length > 0 && (
              <Table
                columns={marketTradeGoodsColumns}
                dataSource={waypoint?.marketTradeGoods.items}
                rowKey={(symbol) => symbol.symbol}
                size="small"
              />
            )}
          {waypoint?.constructionMaterials.items &&
            waypoint.constructionMaterials.items.length > 0 && (
              <Table
                columns={constructionMaterialColumns}
                dataSource={waypoint?.constructionMaterials.items}
                rowKey={(symbol) => symbol.tradeSymbol}
                size="small"
              />
            )}
        </Flex>
      </Flex>
      {waypoint?.marketTransactions.items &&
        waypoint.marketTransactions.items.length > 0 && <Divider />}
      {waypoint?.marketTransactions.items &&
        waypoint.marketTransactions.items.length > 0 && (
          <TransactionTable
            transactions={waypoint?.marketTransactions.items || []}
            reasons={{
              contract: true,
              trade_route_id: true,
              mining: true,
              construction_shipment_id: true,
            }}
          />
        )}
      {(waypoint?.shipyard || waypoint?.shipyardShips) && <Divider />}

      <Flex align="stretch" justify="space-evenly" gap={24} id="shipyard">
        {waypoint?.shipyard && (
          <Descriptions
            bordered
            column={2}
            items={[
              {
                label: "Shipyard",
                key: "shipyard",
                children: (
                  <Space>
                    {waypoint?.shipyard?.waypointSymbol}{" "}
                    <Button
                      loading={loading}
                      onClick={() => {
                        refetch();
                      }}
                    >
                      Reload
                    </Button>
                  </Space>
                ),
              },
              {
                label: "Last Updated",
                key: "createdAt",
                children: new Date(
                  waypoint?.shipyard?.createdAt || "",
                ).toLocaleString(),
              },
              {
                label: "Modifications Fee",
                key: "modificationsFee",
                children: (
                  <MoneyDisplay
                    amount={waypoint?.shipyard?.modificationsFee || 0}
                  />
                ),
              },
              {
                label: "Ships",
                key: "ships",
                children: waypoint?.shipyardShips?.items.length,
              },
            ]}
            // layout="vertical"
            // size="small"
          />
        )}
        {waypoint?.shipyardShips.items &&
          waypoint.shipyardShips.items.length > 0 && (
            <Table
              columns={shipTypesColumns}
              dataSource={waypoint?.shipyardShips.items}
              rowKey={(symbol) => symbol.shipType}
              size="small"
            />
          )}
      </Flex>
      {(oldWaypoint?.shipyard || oldWaypoint?.ship_types) && <Divider />}
      {oldWaypoint?.ships && oldWaypoint.ships.length > 0 && (
        <ShipyardShipTable
          ships={oldWaypoint?.ships}
          onPurchase={(ship) => {
            fetch(`http://${backendUrl}/ship/buy`, {
              method: "POST",
              headers: {
                "Content-Type": "application/json",
              },
              body: JSON.stringify({
                waypointSymbol: waypointID,
                shipType: ship.ship_type,
              }),
            })
              .then((response) => response.json())
              .then(
                (data: {
                  shipSymbol: string;
                  success: boolean;
                  transaction: {
                    agent_symbol: string;
                    price: number;
                    shipType: ShipType;
                    timestamp: string;
                    waypoint_symbol: string;
                  };
                }) => {
                  console.log("Brought Ship", data);
                  message.success(
                    "Brought a " +
                      data.shipSymbol +
                      " for " +
                      data.transaction.price +
                      "$",
                  );
                },
              )
              .then(() => refetch())
              .catch((error) => {
                console.error("Error purchasing ship:", error);
              });
          }}
        />
      )}
      <Divider />
      {(waypoint?.shipyardShipTypes || waypoint?.shipyardShips) && <Divider />}

      {waypoint?.shipyardTransactions.items &&
        waypoint.shipyardTransactions.items.length > 0 && (
          <Table
            columns={shipTransactionColumns}
            dataSource={waypoint?.shipyardTransactions.items}
            rowKey={(symbol) => symbol.id}
            // size="small"

            pagination={{
              showSizeChanger: true,
              pageSizeOptions: ["10", "20", "50", "100", "200", "500", "1000"],
              defaultPageSize: 20,
              showTotal: (total, range) =>
                `${range[0]}-${range[1]} of ${total}`,
            }}
          />
        )}
    </div>
  );
}

export default Waypoint;
