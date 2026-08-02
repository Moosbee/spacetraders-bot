import {
  Button,
  Descriptions,
  DescriptionsProps,
  Divider,
  Flex,
  List,
  Space,
  Table,
  TableProps,
} from "antd";
import { useMemo, useState } from "react";
import { Link, useParams } from "react-router-dom";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import Timer from "../features/Timer/Timer";
import TransactionTable from "../features/TransactionTableOld/TransactionTable";
import WaypointLink from "../features/WaypointLink";

import { useMutation, useQuery } from "@apollo/client/react";
import { backendUrl } from "../data";
import ShipyardShipTable from "../features/ShipyardShipTable/ShipyardShipTable";
import { REPOPULATE_SYSTEMS_WITH_FLEETS_FROM_JUMP_GATE } from "../graphql/mutations";
import { GET_WAYPOINT } from "../graphql/queries";
import {
  ActivityLevel,
  MarketTradeGoodTypeEnum,
  ShipType,
  SupplyLevel,
} from "../models/api";
import { ConstructionMaterial } from "../models/Construction";
import { MarketTrade, MarketTradeGood } from "../models/Market";
import { ShipTransaction, ShipyardShipType } from "../models/Shipyard";
import { WaypointResponse } from "../models/SQLWaypoint";
import { useAppSelector } from "../redux/hooks";
import { selectAllShipsArray } from "../redux/slices/shipSlice";
import { message } from "../utils/antdMessage";

function Waypoint() {
  const { systemID } = useParams();
  const { waypointID } = useParams();

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

  const [oldWaypoint, setWaypoint] = useState<WaypointResponse | null>(null);

  const waypoint = data?.waypoint;

  const ships = useAppSelector(selectAllShipsArray);

  const onSystemsShips = useMemo(() => {
    return ships.filter((ship) => ship.nav.waypoint_symbol === waypointID);
  }, [waypointID, ships]);

  const items: DescriptionsProps["items"] = [
    {
      label: "Symbol",
      key: "symbol",
      children: (
        <WaypointLink waypoint={waypoint?.symbol || ""}>
          {waypoint?.symbol}
        </WaypointLink>
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
      children: `X: ${waypoint?.x} Y: ${waypoint?.y}`,
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
      children: (
        <List
          size="small"
          dataSource={waypoint?.orbitals?.map((orbitals) => (
            <WaypointLink waypoint={orbitals}>{orbitals}</WaypointLink>
          ))}
          renderItem={(item) => <List.Item>{item}</List.Item>}
        ></List>
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

  const marketTradeColumns: TableProps<MarketTrade>["columns"] = [
    {
      title: "Symbol",
      dataIndex: "symbol",
      key: "symbol",
      sorter: (a, b) => a.symbol.localeCompare(b.symbol),
      filters: [
        ...new Set(
          (oldWaypoint?.market_trade_goods || []).map((t) => t.symbol),
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
      filters: Object.values(MarketTradeGoodTypeEnum).map((t) => ({
        text: t,
        value: t,
      })),
      onFilter: (value, record) => record.type === value,
    },
  ];

  const marketTradeGoodsColumns: TableProps<MarketTradeGood>["columns"] = [
    {
      title: "Symbol",
      dataIndex: "symbol",
      key: "symbol",
      sorter: (a, b) => a.symbol.localeCompare(b.symbol),
      filters: [
        ...new Set(
          (oldWaypoint?.market_trade_goods || []).map((t) => t.symbol),
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
      filters: Object.values(MarketTradeGoodTypeEnum).map((t) => ({
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
      dataIndex: "trade_volume",
      key: "trade_volume",

      sorter: (a, b) => a.trade_volume - b.trade_volume,
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
      dataIndex: "purchase_price",
      key: "purchase_price",
      render: (purchasePrice, data) =>
        data.type === "IMPORT" ? (
          <MoneyDisplay amount={purchasePrice} />
        ) : (
          <b>
            <MoneyDisplay amount={purchasePrice} />
          </b>
        ),
      sorter: (a, b) => a.purchase_price - b.purchase_price,
    },
    {
      title: "Sell",
      dataIndex: "sell_price",
      key: "sell_price",
      render: (sellPrice, data) =>
        data.type === "EXPORT" ? (
          <MoneyDisplay amount={sellPrice} />
        ) : (
          <b>
            <MoneyDisplay amount={sellPrice} />
          </b>
        ),
      sorter: (a, b) => a.sell_price - b.sell_price,
    },
    {
      title: "At",
      dataIndex: "created_at",
      key: "created_at",
      render: (date: string) => new Date(date).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.created_at).getTime() - new Date(b.created_at).getTime(),
    },
  ];

  const shipTypesColumns: TableProps<ShipyardShipType>["columns"] = [
    {
      title: "Ship Type",
      dataIndex: "ship_type",
      key: "ship_type",
      sorter: (a, b) => a.ship_type.localeCompare(b.ship_type),
      filters: Object.values(ShipType).map((t) => ({
        text: t,
        value: t,
      })),
      onFilter: (value, record) => record.ship_type === value,
    },
    {
      title: "Created At",
      dataIndex: "created_at",
      key: "created_at",
      render: (date: string) => new Date(date).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.created_at).getTime() - new Date(b.created_at).getTime(),
    },
  ];

  const shipTransactionColumns: TableProps<ShipTransaction>["columns"] = [
    {
      title: "Agent Symbol",
      dataIndex: "agent_symbol",
      key: "agent_symbol",
      sorter: (a, b) => a.agent_symbol.localeCompare(b.agent_symbol),
    },
    {
      title: "Ship Type",
      dataIndex: "ship_type",
      key: "ship_type",
      sorter: (a, b) => a.ship_type.localeCompare(b.ship_type),
      filters: Object.values(ShipType).map((t) => ({
        text: t,
        value: t,
      })),
      onFilter: (value, record) => record.ship_type === value,
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

  const constructionMaterialColumns: TableProps<ConstructionMaterial>["columns"] =
    [
      {
        title: "Trade Symbol",
        dataIndex: "trade_symbol",
        key: "trade_symbol",
        sorter: (a, b) => a.trade_symbol.localeCompare(b.trade_symbol),
      },
      {
        title: "Required",
        dataIndex: "required",
        key: "required",
        sorter: (a, b) => a.required - b.required,
      },
      {
        title: "Fulfilled",
        dataIndex: "fulfilled",
        key: "fulfilled",
        sorter: (a, b) => a.fulfilled - b.fulfilled,
      },
      {
        title: "Last Updated",
        dataIndex: "updated_at",
        key: "updated_at",
        render: (date: string) => new Date(date).toLocaleString(),
        sorter: (a, b) =>
          new Date(a.updated_at).getTime() - new Date(b.updated_at).getTime(),
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
        >
          Reload
        </Button>
        {oldWaypoint?.shipyard && <a href="#shipyard">Shipyard</a>}
        {oldWaypoint?.trade_good_history && (
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
          {oldWaypoint?.market_trades &&
            oldWaypoint.market_trades.length > 0 && (
              <Table
                columns={marketTradeColumns}
                dataSource={oldWaypoint?.market_trades}
                rowKey={(symbol) => symbol.symbol + symbol.waypoint_symbol}
                size="small"
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
          {oldWaypoint?.market_trade_goods &&
            oldWaypoint.market_trade_goods.length > 0 && (
              <Table
                columns={marketTradeGoodsColumns}
                dataSource={oldWaypoint?.market_trade_goods}
                rowKey={(symbol) => symbol.symbol + symbol.waypoint_symbol}
                size="small"
              />
            )}
          {oldWaypoint?.constructions &&
            oldWaypoint.constructions.length > 0 && (
              <Table
                columns={constructionMaterialColumns}
                dataSource={oldWaypoint?.constructions}
                rowKey={(symbol) =>
                  symbol.trade_symbol + symbol.waypoint_symbol
                }
                size="small"
              />
            )}
        </Flex>
      </Flex>
      {oldWaypoint?.transactions && oldWaypoint.transactions.length > 0 && (
        <Divider />
      )}
      {oldWaypoint?.transactions && oldWaypoint.transactions.length > 0 && (
        <TransactionTable
          transactions={oldWaypoint?.transactions || []}
          reasons={{ contract: true, trade_route: true, mining: true }}
        />
      )}
      {(oldWaypoint?.shipyard || oldWaypoint?.ship_types) && <Divider />}

      <Flex align="stretch" justify="space-evenly" gap={24} id="shipyard">
        {oldWaypoint?.shipyard && (
          <Descriptions
            bordered
            column={2}
            items={[
              {
                label: "Shipyard",
                key: "shipyard",
                children: (
                  <Space>
                    {oldWaypoint?.shipyard?.waypoint_symbol}{" "}
                    <Button
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
                key: "last_updated",
                children: new Date(
                  oldWaypoint?.shipyard?.created_at || "",
                ).toLocaleString(),
              },
              {
                label: "Modifications Fee",
                key: "modifications_fee",
                children: (
                  <MoneyDisplay
                    amount={oldWaypoint?.shipyard?.modifications_fee || 0}
                  />
                ),
              },
              {
                label: "Ships",
                key: "ships",
                children: oldWaypoint?.ship_types?.length,
              },
            ]}
            // layout="vertical"
            // size="small"
          />
        )}
        {oldWaypoint?.ship_types && oldWaypoint.ship_types.length > 0 && (
          <Table
            columns={shipTypesColumns}
            dataSource={oldWaypoint?.ship_types}
            rowKey={(symbol) => symbol.ship_type + symbol.shipyard_id}
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
                    ship_type: ShipType;
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
      {(oldWaypoint?.ships || oldWaypoint?.ship_types) && <Divider />}

      {oldWaypoint?.ship_transactions &&
        oldWaypoint.ship_transactions.length > 0 && (
          <Table
            columns={shipTransactionColumns}
            dataSource={oldWaypoint?.ship_transactions}
            rowKey={(symbol) =>
              symbol.agent_symbol +
              symbol.waypoint_symbol +
              symbol.ship_type +
              symbol.timestamp
            }
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
