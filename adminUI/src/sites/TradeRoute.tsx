import { useQuery } from "@apollo/client/react";
import {
  Button,
  Col,
  Descriptions,
  DescriptionsProps,
  Divider,
  Flex,
  Result,
  Row,
  Space,
  Spin,
  Table,
  TableProps,
} from "antd";
import { Link, useParams } from "react-router-dom";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import TransactionTable from "../features/TransactionTable/TransactionTable";
import WaypointLink from "../features/WaypointLink";
import WaypointTable from "../features/WaypointTable/WaypointTable";
import { GetTradeRouteQuery, ShipmentStatus } from "../gql/graphql";
import { GET_TRADE_ROUTE } from "../graphql/queries";
import { cn } from "../utils/utils";

type TradeRouteData = GetTradeRouteQuery["tradeRoute"];

type TradeGood = NonNullable<TradeRouteData["purchaseMarketTradeGood"]>;

const statusColor = (status: ShipmentStatus) =>
  status === ShipmentStatus.Delivered
    ? "green"
    : status === ShipmentStatus.Failed
      ? "red"
      : status === ShipmentStatus.InTransit
        ? "yellow"
        : undefined;

function TradeRoute() {
  const { routeId } = useParams();

  const id = Number.parseInt(routeId ?? "", 10);

  const { loading, error, data, refetch } = useQuery(GET_TRADE_ROUTE, {
    variables: { routeId: id },
    skip: Number.isNaN(id),
  });

  if (error) {
    return (
      <div style={{ padding: "24px 24px" }}>
        <PageTitle title={`Trade Route ${routeId}`} />
        <Result
          status="error"
          title="Trade Route Error"
          subTitle={error.message}
          extra={[
            <Button key="retry" type="primary" onClick={() => refetch()}>
              Try Again
            </Button>,
          ]}
        />
      </div>
    );
  }

  const tradeRoute = data?.tradeRoute;

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title={`Trade Route ${routeId}`} />
      <Spin spinning={loading}>
        <Space wrap>
          <h2>Trade Route {tradeRoute?.id ?? routeId}</h2>
          <Button onClick={() => refetch()} loading={loading}>
            Reload
          </Button>
        </Space>

        <Divider />

        {tradeRoute ? (
          <TradeRouteDetails tradeRoute={tradeRoute} />
        ) : (
          !loading && (
            <Result
              status="404"
              title="Trade Route not found"
              subTitle={`No trade route with id ${routeId}`}
            />
          )
        )}
      </Spin>
    </div>
  );
}

function TradeRouteDetails({ tradeRoute }: { tradeRoute: TradeRouteData }) {
  const summary = tradeRoute.marketTransactionSummary;
  const purchaseGood = tradeRoute.purchaseMarketTradeGood;
  const sellGood = tradeRoute.sellMarketTradeGood;
  const purchaseGoodCurrent = tradeRoute.purchaseMarketTradeGoodCurrent;
  const sellGoodCurrent = tradeRoute.sellMarketTradeGoodCurrent;

  const predictedPurchaseCost =
    (purchaseGood?.purchasePrice ?? 0) * tradeRoute.tradeVolume;
  const predictedTotalCost =
    predictedPurchaseCost + (tradeRoute.estimatedFuel ?? 0);
  const predictedSellRevenue =
    (sellGood?.sellPrice ?? 0) * tradeRoute.tradeVolume;
  const predictedProfit =
    predictedSellRevenue -
    predictedPurchaseCost -
    (tradeRoute.estimatedFuel ?? 0);
  const actualPurchaseCost = summary.allExpenses ?? 0;
  const actualFuelCost = summary.fuelExpenses ?? 0;
  const actualExpenses = actualPurchaseCost + actualFuelCost;
  const actualIncome = summary.allIncome ?? 0;
  const actualProfit = actualIncome - actualExpenses;

  const infoItems: DescriptionsProps["items"] = [
    {
      key: "symbol",
      label: "Trade Symbol",
      children: (
        <Link to={`/supplyChain/${tradeRoute.symbol}`}>
          {tradeRoute.symbol}
        </Link>
      ),
    },
    {
      key: "status",
      label: "Status",
      children: (
        <span style={{ color: statusColor(tradeRoute.status) }}>
          {tradeRoute.status}
        </span>
      ),
    },
    {
      key: "tradeMode",
      label: "Trade Mode",
      children: tradeRoute.tradeMode,
    },
    {
      key: "createdAt",
      label: "Created At",
      children: new Date(tradeRoute.createdAt).toLocaleString(),
    },
    {
      key: "ship",
      label: "Ship",
      children: (
        <Link to={`/ships/${tradeRoute.shipSymbol}`}>
          {tradeRoute.shipSymbol}
        </Link>
      ),
    },
    {
      key: "tradeVolume",
      label: "Trade Volume",
      children: tradeRoute.tradeVolume,
    },
    {
      key: "purchaseWaypoint",
      label: "Purchase Waypoint",
      children: (
        <WaypointLink waypoint={tradeRoute.PurchaseWaypointSymbol}>
          {tradeRoute.PurchaseWaypointSymbol}
        </WaypointLink>
      ),
    },
    {
      key: "sellWaypoint",
      label: "Sell Waypoint",
      children: (
        <WaypointLink waypoint={tradeRoute.SellWaypointSymbol}>
          {tradeRoute.SellWaypointSymbol}
        </WaypointLink>
      ),
    },
    {
      key: "purchaseTradeGoodId",
      label: "Purchase Trade Good ID",
      children: tradeRoute.purchaseTradeGoodId ?? "N/A",
    },
    {
      key: "sellTradeGoodId",
      label: "Sell Trade Good ID",
      children: tradeRoute.sellTradeGoodId ?? "N/A",
    },
    {
      key: "estimatedFuel",
      label: "Estimated Fuel",
      children: <MoneyDisplay amount={tradeRoute.estimatedFuel ?? 0} />,
    },
    {
      key: "reservedFund",
      label: "Reserved Fund ID",
      children: tradeRoute.reservedFund ?? "N/A",
    },
    {
      key: "fleet",
      label: "Fleet",
      children: tradeRoute.fleet ? (
        <Link to={`/fleets/${tradeRoute.fleet.id}`}>
          {tradeRoute.fleet.fleetType}_{tradeRoute.fleet.id} (
          {tradeRoute.fleet.active ? "Active" : "Inactive"})
        </Link>
      ) : (
        (tradeRoute.fleetId ?? "N/A")
      ),
    },
    {
      key: "assignment",
      label: "Assignment",
      children: tradeRoute.assignmentId ?? "N/A",
    },
  ];

  const moneyItems: DescriptionsProps["items"] = [
    {
      key: "predictedPurchaseCost",
      label: "Predicted Purchase Cost",
      children: <MoneyDisplay amount={predictedPurchaseCost} />,
    },
    {
      key: "actualPurchaseCost",
      label: "Actual Purchase Cost",
      children: <MoneyDisplay amount={actualPurchaseCost} />,
    },
    {
      key: "predictedFuelCost",
      label: "Predicted Fuel Cost",
      children: <MoneyDisplay amount={tradeRoute.estimatedFuel ?? 0} />,
    },
    {
      key: "actualFuelCost",
      label: "Actual Fuel Cost",
      children: <MoneyDisplay amount={actualFuelCost} />,
    },
    {
      key: "predictedTotalCost",
      label: "Predicted Total Cost",
      children: <MoneyDisplay amount={predictedTotalCost} />,
    },
    {
      key: "actualExpenses",
      label: "Actual Expenses",
      children: <MoneyDisplay amount={actualExpenses} />,
    },
    {
      key: "PredictedSellRevenue",
      label: "Predicted Sell Revenue",
      children: <MoneyDisplay amount={predictedSellRevenue} />,
    },
    {
      key: "actualIncome",
      label: "Actual Revenue",
      children: <MoneyDisplay amount={actualIncome} />,
    },
    {
      key: "predictedProfit",
      label: "Predicted Profit",
      children: (
        <MoneyDisplay
          amount={predictedProfit}
          className={cn(predictedProfit > 0 ? "text-current" : "text-red-600")}
        />
      ),
    },
    {
      key: "actualProfit",
      label: "Profit",
      children: (
        <MoneyDisplay
          amount={actualProfit}
          className={cn(actualProfit > 0 ? "text-current" : "text-red-600")}
        />
      ),
    },
    {
      key: "purchaseTransactions",
      label: "Purchase Transactions",
      children: summary.purchaseTransactions ?? 0,
    },
    {
      key: "sellTransactions",
      label: "Sell Transactions",
      children: summary.sellTransactions ?? 0,
    },
    {
      key: "purchaseUnits",
      label: "Purchase Units",
      children: summary.purchaseUnits ?? 0,
    },
    {
      key: "sellUnits",
      label: "Sell Units",
      children: summary.sellUnits ?? 0,
    },
    {
      key: "fuelTransactions",
      label: "Fuel Transactions",
      children: summary.fuelPurchaseTransactions ?? 0,
    },
    {
      key: "fuelUnits",
      label: "Fuel Units",
      children: summary.fuelPurchaseUnits ?? 0,
    },
    {
      key: "allPurchaseTransactions",
      label: "All Purchase Transactions",
      children: summary.allPurchaseTransactions ?? 0,
    },
    {
      key: "allSellTransactions",
      label: "All Sell Transactions",
      children: summary.allSellTransactions ?? 0,
    },
    {
      key: "allPurchaseUnits",
      label: "All Purchase Units",
      children: summary.allPurchaseUnits ?? 0,
    },
    {
      key: "allSellUnits",
      label: "All Sell Units",
      children: summary.allSellUnits ?? 0,
    },
  ];

  const reservationItems: DescriptionsProps["items"] = tradeRoute.reservation
    ? [
        {
          key: "id",
          label: "ID",
          children: tradeRoute.reservation.id,
        },
        {
          key: "amount",
          label: "Amount",
          children: <MoneyDisplay amount={tradeRoute.reservation.amount} />,
        },
        {
          key: "actualAmount",
          label: "Actual Amount",
          children: (
            <MoneyDisplay amount={tradeRoute.reservation.actualAmount} />
          ),
        },
        {
          key: "status",
          label: "Status",
          children: tradeRoute.reservation.status,
        },
        {
          key: "createdAt",
          label: "Created At",
          children: new Date(tradeRoute.reservation.createdAt).toLocaleString(),
        },
        {
          key: "updatedAt",
          label: "Updated At",
          children: new Date(tradeRoute.reservation.updatedAt).toLocaleString(),
        },
      ]
    : [];

  const goodColumns = (
    title: string,
    type: "Purchase" | "Sell",
  ): TableProps<TradeGood>["columns"] => [
    {
      title: "Created At",
      dataIndex: "createdAt",
      key: "createdAt",
      render: (date: string) => new Date(date).toLocaleString(),
    },
    {
      title: `${title} Waypoint`,
      dataIndex: "waypointSymbol",
      key: "waypointSymbol",
      render: (symbol: string) =>
        symbol ? (
          <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
        ) : (
          "N/A"
        ),
    },
    {
      title: "Type",
      dataIndex: "type",
      key: "type",
    },
    {
      title: "Supply",
      dataIndex: "supply",
      key: "supply",
    },
    {
      title: "Activity",
      dataIndex: "activity",
      key: "activity",
      render: (value: string | null | undefined) => value ?? "N/A",
    },
    {
      title: "Trade Volume",
      dataIndex: "tradeVolume",
      key: "tradeVolume",
      align: "right",
    },
    {
      title: "Purchase Price",
      dataIndex: "purchasePrice",
      key: "purchasePrice",
      align: "right",
      render: (value: number) => (
        <MoneyDisplay
          amount={value}
          className={type === "Purchase" ? "font-bold" : ""}
        />
      ),
    },
    {
      title: "Sell Price",
      dataIndex: "sellPrice",
      key: "sellPrice",
      align: "right",
      render: (value: number) => (
        <MoneyDisplay
          amount={value}
          className={type === "Sell" ? "font-bold" : ""}
        />
      ),
    },
  ];

  const waypoints = [
    ...(tradeRoute.purchaseWaypoint ? [tradeRoute.purchaseWaypoint] : []),
    ...(tradeRoute.sellWaypoint ? [tradeRoute.sellWaypoint] : []),
  ];

  return (
    <Flex gap={12} vertical>
      <Row gutter={[8, 8]}>
        <Col span={12}>
          <Descriptions
            bordered
            size="small"
            column={2}
            title="Trade Route Info"
            items={infoItems}
          />
        </Col>
        <Col span={12}>
          <Descriptions
            bordered
            size="small"
            column={2}
            title="Financial Summary"
            items={moneyItems}
          />
        </Col>
      </Row>

      {tradeRoute.reservation && (
        <>
          <Descriptions
            bordered
            size="small"
            column={3}
            title="Reservation"
            items={reservationItems}
          />
        </>
      )}

      <Divider />
      <Row gutter={[8, 8]}>
        <Col span={12}>
          <Table
            size="small"
            title={() => "Purchase Market Trade Good"}
            columns={goodColumns("Purchase", "Purchase")}
            dataSource={purchaseGood ? [purchaseGood] : []}
            rowKey={(record) => `${record.id}-purchase`}
            pagination={false}
            locale={{ emptyText: <span>No purchase market trade good</span> }}
          />
        </Col>
        <Col span={12}>
          <Table
            size="small"
            title={() => "Sell Market Trade Good"}
            columns={goodColumns("Sell", "Sell")}
            dataSource={sellGood ? [sellGood] : []}
            rowKey={(record) => `${record.id}-sell`}
            pagination={false}
            locale={{ emptyText: <span>No sell market trade good</span> }}
          />
        </Col>
      </Row>

      {purchaseGoodCurrent || sellGoodCurrent ? (
        <Row gutter={[8, 8]}>
          <Col span={12}>
            <Table
              size="small"
              title={() => "Current Purchase Market Trade Good"}
              columns={goodColumns("Purchase", "Purchase")}
              dataSource={purchaseGoodCurrent ? [purchaseGoodCurrent] : []}
              rowKey={(record) => `${record.id}-purchase-current`}
              pagination={false}
            />
          </Col>
          <Col span={12}>
            <Table
              size="small"
              title={() => "Current Sell Market Trade Good"}
              columns={goodColumns("Sell", "Sell")}
              dataSource={sellGoodCurrent ? [sellGoodCurrent] : []}
              rowKey={(record) => `${record.id}-sell-current`}
              pagination={false}
            />
          </Col>
        </Row>
      ) : null}

      <Divider />
      <WaypointTable waypoints={waypoints} pagination={false} />

      <Divider />
      <TransactionTable
        transactions={tradeRoute.transactions.items}
        reasons={{
          contract: true,
          trade_route_id: false,
          mining: true,
          construction_shipment_id: true,
        }}
      />
    </Flex>
  );
}

export default TradeRoute;
