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
import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import TransactionTable from "../features/TransactionTable/TransactionTable";
import WaypointLink from "../features/WaypointLink";
import {
  ActivityLevel,
  MarketTradeGoodTypeEnum,
  SupplyLevel,
} from "../models/api";
import {
  MarketTrade,
  MarketTradeGood,
  WaypointResponse,
} from "../models/SQLWaypoint";
import { backendUrl } from "../store";

function Waypoint() {
  const { systemID } = useParams();
  const { waypointID } = useParams();

  const [waypoint, setWaypoint] = useState<WaypointResponse | null>(null);

  useEffect(() => {
    fetch(`http://${backendUrl}/waypoints/${waypointID}`)
      .then((response) => response.json())
      .then((data) => {
        console.log("waypoint", data);

        setWaypoint(data);
      });
  }, [waypointID]);

  const items: DescriptionsProps["items"] = [
    {
      label: "Symbol",
      key: "symbol",
      children: (
        <WaypointLink waypoint={waypoint?.waypoint.symbol || ""}>
          {waypoint?.waypoint.symbol}
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
      children: waypoint?.waypoint?.waypoint_type,
    },
    {
      label: "Coordinates",
      key: "coordinates",
      children: `X: ${waypoint?.waypoint?.x} Y: ${waypoint?.waypoint?.y}`,
    },
    {
      key: "chart",
      label: "Chart",
      children: (
        <p>
          By: {waypoint?.waypoint.charted_by} <br />
          On:{" "}
          {new Date(
            waypoint?.waypoint.charted_on ? waypoint?.waypoint.charted_on : 0
          ).toLocaleDateString()}
        </p>
      ),
    },
    {
      label: "Faction",
      key: "faction",
      children: waypoint?.waypoint?.faction || "None",
    },

    {
      label: "Orbits",
      key: "orbits",
      children: waypoint?.waypoint?.orbits || "None",
    },
    {
      label: "Orbitals",
      key: "orbitals",
      children: (
        <List
          size="small"
          dataSource={waypoint?.waypoint?.orbitals?.map((orbitals) => (
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
        (waypoint?.waypoint?.modifiers?.length || 0) > 0 ? (
          <List
            size="small"
            dataSource={waypoint?.waypoint.modifiers?.map((modifier) => (
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
      children: (
        <List
          size="small"
          dataSource={waypoint?.waypoint.traits.map((trait) => (
            <span>{trait}</span>
          ))}
          renderItem={(item) => <List.Item>{item}</List.Item>}
        ></List>
      ),
    },
  ];

  if (waypoint?.waypoint.is_under_construction) {
    items.push({
      key: "is_under_construction",
      label: "Under Construction",
      children: (
        <p>{waypoint?.waypoint.is_under_construction ? "Yes" : "No"}</p>
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
        ...new Set(waypoint?.market_trade_goods.map((t) => t.symbol)),
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
        ...new Set(waypoint?.market_trade_goods.map((t) => t.symbol)),
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

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title={`Waypoint ${waypointID}`} />
      <Space>
        <h2>
          Waypoint {waypointID} in {systemID}
        </h2>
        <Button
          onClick={() => {
            fetch(`http://${backendUrl}/waypoints/${waypointID}`)
              .then((response) => response.json())
              .then((data) => {
                console.log("waypoint", data);

                setWaypoint(data);
              });
          }}
        >
          Reload
        </Button>
      </Space>
      <Flex align="stretch" justify="flex-start" gap={24}>
        <Descriptions
          bordered
          column={6}
          items={items}
          layout="vertical"
          size="small"
        />
        <Table
          columns={marketTradeColumns}
          dataSource={waypoint?.market_trades}
          size="small"
        />
        <Table
          columns={marketTradeGoodsColumns}
          dataSource={waypoint?.market_trade_goods}
          size="small"
        />
      </Flex>
      <Divider />
      <p>{waypoint?.market_trade_goods.map((t) => t.symbol + " ")}</p>
      <p>{waypoint?.market_trades.map((t) => t.symbol)}</p>
      <TransactionTable
        transactions={waypoint?.transactions || []}
        reasons={{ contract: true, trade_route: true, mining: true }}
      />
    </div>
  );
}

export default Waypoint;
