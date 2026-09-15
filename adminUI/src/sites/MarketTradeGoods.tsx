import { useQuery } from "@apollo/client/react";
import {
  Button,
  Card,
  Col,
  Divider,
  Result,
  Row,
  Space,
  Spin,
  Table,
  TableProps,
} from "antd";
import { Link } from "react-router-dom";
import BoxPlot, { BoxPlotDatum } from "../features/BoxPlot/BoxPlot";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import WaypointLink from "../features/WaypointLink";
import {
  ActivityLevel,
  AllLatestMarketTradeGoodsQuery,
  MarketTradeGoodType,
  SupplyLevel,
} from "../gql/graphql";
import { GET_ALL_LATEST_MARKET_TRADE_GOODS } from "../graphql/queries";

type MarketTradeGood =
  AllLatestMarketTradeGoodsQuery["marketTradeGoods"]["items"][number];

export default function MarketTradeGoods() {
  const { loading, error, data, dataState, refetch } = useQuery(
    GET_ALL_LATEST_MARKET_TRADE_GOODS,
  );

  if (error) {
    return (
      <div style={{ padding: "24px 24px" }}>
        <PageTitle title="Market Trade Goods" />
        <Result
          status="error"
          title="Error"
          subTitle={`Error: ${error.message}`}
          extra={[
            <Button key="retry" type="primary" onClick={() => refetch()}>
              Try Again
            </Button>,
          ]}
        />
      </div>
    );
  }

  const items = data?.marketTradeGoods.items ?? [];

  const boxData = (priceKey: "purchasePrice" | "sellPrice"): BoxPlotDatum[] => {
    const groups = new Map<string, number[]>();
    items.forEach((item) => {
      const list = groups.get(item.symbol) ?? [];
      list.push(item[priceKey]);
      groups.set(item.symbol, list);
    });
    return [...groups.entries()]
      .map(([label, values]) => ({ label, values }))
      .sort((a, b) => a.label.localeCompare(b.label));
  };

  const columns: TableProps<MarketTradeGood>["columns"] = [
    {
      title: "ID",
      dataIndex: "id",
      key: "id",
      sorter: (a, b) => a.id - b.id,
      defaultSortOrder: "descend",
    },
    {
      title: "Symbol",
      dataIndex: "symbol",
      key: "symbol",
      render: (symbol) => <Link to={`/supplyChain/${symbol}`}>{symbol}</Link>,
      sorter: (a, b) => a.symbol.localeCompare(b.symbol),
    },
    {
      title: "Waypoint",
      dataIndex: "waypointSymbol",
      key: "waypointSymbol",
      render: (waypoint) => (
        <WaypointLink waypoint={waypoint}>{waypoint}</WaypointLink>
      ),
      sorter: (a, b) => a.waypointSymbol.localeCompare(b.waypointSymbol),
    },
    {
      title: "Type",
      dataIndex: "type",
      key: "type",
      filters: Object.values(MarketTradeGoodType).map((type) => ({
        text: type,
        value: type,
      })),
      onFilter: (value, record) => record.type === value,
      sorter: (a, b) => a.type.localeCompare(b.type),
    },
    {
      title: "Trade Volume",
      dataIndex: "tradeVolume",
      key: "tradeVolume",
      align: "right",
      sorter: (a, b) => a.tradeVolume - b.tradeVolume,
    },
    {
      title: "Supply",
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
      render: (activity) => activity ?? "N/A",
      filters: Object.values(ActivityLevel).map((activity) => ({
        text: activity,
        value: activity,
      })),
      onFilter: (value, record) => record.activity === value,
      sorter: (a, b) => (a.activity ?? "").localeCompare(b.activity ?? ""),
    },
    {
      title: "Purchase Price",
      dataIndex: "purchasePrice",
      key: "purchasePrice",
      align: "right",
      render: (price) => <MoneyDisplay amount={price} />,
      sorter: (a, b) => a.purchasePrice - b.purchasePrice,
    },
    {
      title: "Sell Price",
      dataIndex: "sellPrice",
      key: "sellPrice",
      align: "right",
      render: (price) => <MoneyDisplay amount={price} />,
      sorter: (a, b) => a.sellPrice - b.sellPrice,
    },
    {
      title: "Created At",
      dataIndex: "createdAt",
      key: "createdAt",
      render: (date) => new Date(date).toLocaleString(),
      sorter: (a, b) => a.createdAt.localeCompare(b.createdAt),
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="Market Trade Goods" />
      <Space>
        <h1>Market Trade Goods {items.length}</h1>
        <Button onClick={() => refetch()}>Refresh</Button>
        <Spin spinning={loading || dataState !== "complete"} />
      </Space>
      <Row gutter={16}>
        <Col xs={24} xl={12}>
          <Card variant="borderless" title="Purchase Price by Trade Symbol">
            <BoxPlot data={boxData("purchasePrice")} />
          </Card>
        </Col>
        <Col xs={24} xl={12}>
          <Card variant="borderless" title="Sell Price by Trade Symbol">
            <BoxPlot data={boxData("sellPrice")} />
          </Card>
        </Col>
      </Row>
      <Divider />
      <Table<MarketTradeGood>
        dataSource={items}
        rowKey="id"
        columns={columns}
        pagination={{
          showSizeChanger: true,
          pageSizeOptions: ["10", "20", "50", "100", "200", "500", "1000"],
          defaultPageSize: 10,
          showTotal: (total, range) => `${range[0]}-${range[1]} of ${total}`,
        }}
      />
    </div>
  );
}
