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
  Result,
  Row,
  Space,
  Spin,
  Table,
  TableProps,
  theme,
  Typography,
} from "antd";
import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import FaIcon from "../features/FontAwsome/FaIcon";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import RoleRenderer from "../features/RoleRenderer/RoleRenderer";
import ShipCapabilityRadar from "../features/ShipCapabilityRadar/ShipCapabilityRadar";
import ShipControls from "../features/ShipControl/ShipControls";
import ShipComponents from "../features/ShipInfo/ShipComponents";
import Timer from "../features/Timer/Timer";
import WaypointLink from "../features/WaypointLink";
import {
  GetShipQuery,
  ShipmentStatus,
  TradeMode,
  TradeSymbol,
  WaypointTraitSymbol,
  WaypointType,
} from "../gql/graphql";
import { GET_SHIP } from "../graphql/queries";
import { useAppDispatch, useAppSelector } from "../redux/hooks";
import {
  selectSelectedShipSymbol,
  setSelectedShipSymbol,
} from "../redux/slices/mapSlice";
import { cn } from "../utils/utils";

type ShipData = GetShipQuery["ship"];

function Ship() {
  const { shipID } = useParams();

  const { loading, error, data, refetch } = useQuery(GET_SHIP, {
    variables: { shipSymbol: shipID || "" },
  });

  // The live websocket model still drives the manual control panel.
  const ship = data?.ship;

  if (error) {
    return (
      <Result
        status="error"
        title="Ship Error"
        subTitle={error.message}
        extra={[
          <Button key="retry" type="primary" onClick={() => refetch()}>
            Try Again
          </Button>,
        ]}
      />
    );
  }

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title={`Ship ${shipID}`} />
      <Spin spinning={loading}>
        <Space wrap>
          <h2>Ship {ship?.symbol ?? shipID}</h2>
          <Button onClick={() => refetch()} loading={loading}>
            Reload
          </Button>
        </Space>

        <Divider />

        {ship ? (
          <ShipDetails ship={ship} />
        ) : (
          !loading && (
            <Result
              status="404"
              title="Ship not found"
              subTitle={`No ship with symbol ${shipID}`}
            />
          )
        )}
      </Spin>
    </div>
  );
}

function ShipDetails({ ship }: { ship: ShipData }) {
  const selectedShip = useAppSelector(selectSelectedShipSymbol);
  const {
    token: { colorText },
  } = theme.useToken();
  const dispatch = useAppDispatch();
  const autoPilot = ship.nav.autoPilot;
  const autoPilotOrigin = autoPilot
    ? autoPilot.originSystemSymbol === autoPilot.destinationSystemSymbol
      ? autoPilot.originSymbol.replace(autoPilot.originSystemSymbol + "-", "")
      : autoPilot.originSymbol
    : null;
  const autoPilotDestination = autoPilot
    ? autoPilot.originSystemSymbol === autoPilot.destinationSystemSymbol
      ? autoPilot.destinationSymbol.replace(
          autoPilot.destinationSystemSymbol + "-",
          "",
        )
      : autoPilot.destinationSymbol
    : null;

  const color = colorText;
  const shipIcon =
    ship.nav.status === "IN_ORBIT" ? (
      <FaIcon type="solid" icon="fa-rocket" />
    ) : ship.nav.status === "DOCKED" ? (
      <FaIcon type="solid" icon="fa-rocket" />
    ) : (
      <FaIcon type="solid" icon="fa-rocket-launch" />
    );

  const infoItems: DescriptionsProps["items"] = [
    {
      key: "symbol",
      label: "Symbol",
      children: (
        <button
          onClick={() => {
            if (selectedShip === ship?.symbol) {
              dispatch(setSelectedShipSymbol(undefined));
              return;
            }
            dispatch(setSelectedShipSymbol(ship?.symbol));
          }}
          className="cursor-pointer flex justify-stretch items-center w-full py-1"
        >
          <div
            style={{ color: color }}
            className="h-6 w-6 flex justify-center items-center text-xl mr-2"
          >
            <span
              className="absolute"
              style={{
                boxShadow:
                  selectedShip == ship?.symbol
                    ? "0px 0px calc(0.8 * 1.25rem) calc(0.6 * 1.25rem) color-mix(in srgb, currentColor 80%, #fff 20%)"
                    : "",
              }}
            ></span>
            {shipIcon}
          </div>
          {ship?.symbol}
        </button>
      ),
    },
    {
      label: "Active",
      key: "active",
      children: ship.status ? "Yes*" : "No*",
    },
    {
      key: "system",
      label: "System",
      children: (
        <Link to={`/system/${ship.nav.systemSymbol}`}>
          {ship.nav.systemSymbol}
        </Link>
      ),
    },
    {
      key: "waypoint",
      label: "Waypoint",
      children: (
        <WaypointLink waypoint={ship.nav.waypointSymbol}>
          {ship.nav.waypointSymbol}
        </WaypointLink>
      ),
    },
    {
      key: "status",
      label: "Status",
      children: <RoleRenderer status={ship.status} />,
      span: 2,
    },
    { key: "flightMode", label: "Flight Mode", children: ship.nav.flightMode },
    { key: "engineSpeed", label: "Engine Speed", children: ship.engineSpeed },
    {
      key: "waiting",
      label: "Waiting",
      children: `API: ${ship.status.waitingForApi ? "Yes" : "No"} · Manager: ${
        ship.status.waitingForManager ? "Yes" : "No"
      }`,
    },
    {
      key: "registrationRole",
      label: "Registration Role",
      children: ship.registrationRole,
    },
    {
      key: "navStatus",
      label: "Nav Status",
      span: 2,
      children: (
        <span>
          {ship.nav.status}
          {ship.nav.status === "IN_TRANSIT" && (
            <>
              {" "}
              (<Timer time={ship.nav.route.arrival} />)
              <ShipNavProgress
                departureTime={ship.nav.route.departureTime}
                arrival={ship.nav.route.arrival}
              />
              <br />
              {ship.nav.route.originSymbol} -{">"}{" "}
              {ship.nav.route.destinationSymbol}
            </>
          )}
        </span>
      ),
    },
    {
      key: "cargo",
      label: "Cargo",
      children: (
        <Popover
          content={
            <Flex vertical>
              {ship.cargo.inventory.map((item) => (
                <Flex gap={6} justify="space-between" key={item.symbol}>
                  <span>{item.symbol}</span>
                  <span>{item.units}</span>
                </Flex>
              ))}
            </Flex>
          }
        >
          {`${ship.cargo.units} / ${ship.cargo.capacity}`}
        </Popover>
      ),
    },
    {
      key: "fuel",
      label: "Fuel",
      children: `${ship.fuel.current} / ${ship.fuel.capacity}`,
    },
    {
      key: "autoPilot",
      label: "Auto Pilot",
      span: 2,

      children: autoPilot ? (
        <span>
          {autoPilotOrigin} -{">"} {autoPilotDestination}
          <br />
          <ShipNavProgress
            departureTime={autoPilot.departureTime}
            arrival={autoPilot.arrival}
          />
          <br />
          (<Timer time={autoPilot.arrival} />)
          <br />
          <Popover
            title="Autopilot Route Totals"
            content={
              <Flex vertical gap={2}>
                <span>
                  Total Distance: {Math.round(autoPilot.route.totalDistance)}
                </span>
                <span>Total Fuel: {autoPilot.route.totalFuel}</span>
                <span>
                  Total Anti-Matter: {autoPilot.route.totalAntiMatter}
                </span>
                <span>
                  Total Fuel Cost:{" "}
                  <MoneyDisplay amount={autoPilot.route.totalFuelCost} />
                </span>
                <span>
                  Total Anti-Matter Cost:{" "}
                  <MoneyDisplay amount={autoPilot.route.totalAntiMatterCost} />
                </span>
                <span>
                  Total Cost:{" "}
                  <MoneyDisplay amount={autoPilot.route.totalCost} />
                </span>
                <span>
                  Total Travel Time: {autoPilot.route.totalTravelTime}s
                </span>
                <span>
                  Total Jump Cooldown: {autoPilot.route.totalJumpCooldownTime}s
                </span>
                <span>
                  Total API Requests: {autoPilot.route.totalApiRequests}
                </span>
              </Flex>
            }
          >
            Distance: {Math.round(autoPilot.distance)} · Travel:{" "}
            {autoPilot.travelTime}s · Hops: {autoPilot.route.connections.length}
          </Popover>
        </span>
      ) : (
        <span>No Auto Pilot</span>
      ),
    },
    {
      key: "cooldownSeconds",
      label: "Cooldown",
      children: (
        <span>
          {ship.cooldown ?? "N/A"}
          <br />
          {ship.cooldownExpiration && <Timer time={ship.cooldownExpiration} />}
        </span>
      ),
    },
    {
      key: "conditions",
      label: "Conditions",
      children: <Conditions conditions={ship.conditions} />,
      span: 1,
    },

    {
      key: "autopilotRoute",
      label: "Autopilot Route",
      children: ship.nav.autoPilot?.route.connections?.length ? (
        <List
          size="small"
          split={false}
          className="max-h-64 scroll-auto overflow-y-scroll"
          dataSource={ship.nav.autoPilot?.route.connections}
          renderItem={(item) => {
            const mark = ship.nav.waypointSymbol === item.endSymbol;

            if ("navMode" in item) {
              return (
                <List.Item>
                  <Popover
                    title={`${item.__typename?.replace("Connection", "") ?? "Nav"} Connection`}
                    content={
                      <Flex vertical gap={2}>
                        <span>From: {item.startSymbol}</span>
                        <span>To: {item.endSymbol}</span>
                        <span>Nav Mode: {item.navMode}</span>
                        <span>Distance: {Math.round(item.distance)}</span>
                        <span>Travel Time: {item.travelTime}s</span>
                        <span>Refuel: {item.refuel.fuelNeeded}</span>
                      </Flex>
                    }
                  >
                    <Typography.Text mark={mark}>
                      {item.navMode}{" "}
                      <WaypointLink waypoint={item.startSymbol}>
                        {item.startSymbol}
                      </WaypointLink>{" "}
                      -{">"}{" "}
                      <WaypointLink waypoint={item.endSymbol}>
                        {item.endSymbol}
                      </WaypointLink>{" "}
                      ({item.travelTime}s)
                    </Typography.Text>
                  </Popover>
                </List.Item>
              );
            }

            return (
              <List.Item>
                <Popover
                  title="Jump Connection"
                  content={
                    <Flex vertical gap={2}>
                      <span>From: {item.startSymbol}</span>
                      <span>To: {item.endSymbol}</span>
                      <span>Distance: {Math.round(item.distance)}</span>
                      <span>Cooldown: {item.cooldownTime}s</span>
                    </Flex>
                  }
                >
                  <Typography.Text mark={mark}>
                    Jump{" "}
                    <WaypointLink waypoint={item.startSymbol}>
                      {item.startSymbol}
                    </WaypointLink>{" "}
                    -{">"}{" "}
                    <WaypointLink waypoint={item.endSymbol}>
                      {item.endSymbol}
                    </WaypointLink>{" "}
                    ({Math.round(item.distance)})
                  </Typography.Text>
                </Popover>
              </List.Item>
            );
          }}
        />
      ) : (
        "N/A"
      ),
    },
  ];

  const inventoryColumns: TableProps<
    ShipData["cargo"]["inventory"][number]
  >["columns"] = [
    {
      title: "Symbol",
      dataIndex: "symbol",
      key: "symbol",
      sorter: (a, b) => a.symbol.localeCompare(b.symbol),
      filters: Object.values(TradeSymbol).map((symbol) => ({
        text: symbol,
        value: symbol,
      })),
      onFilter: (value, record) => record.symbol === value,
    },
    {
      title: "Units",
      dataIndex: "units",
      key: "units",
      align: "right",
      sorter: (a, b) => a.units - b.units,
    },
  ];

  const renderAssignment = (assignment: ShipData["status"]["assignment"]) =>
    assignment ? (
      <span>
        disabled: {assignment.disabled ? "Yes" : "No"}
        <br />
        id: {assignment.id} fleet: {assignment.fleetId} priority:{" "}
        {assignment.priority}
        <br /> rangeMin: {assignment.rangeMin} cargoMin: {assignment.cargoMin}
        <br /> maxPurchase:{" "}
        <MoneyDisplay amount={assignment.maxPurchasePrice} /> credits:{" "}
        <MoneyDisplay amount={assignment.creditsThreshold} />
        <br /> siphon: {assignment.siphon ? "Yes" : "No"} warp:{" "}
        {assignment.warpDrive ? "Yes" : "No"}
        <br />
        extractor: {assignment.extractor ? "Yes" : "No"} survey:{" "}
        {assignment.survey ? "Yes" : "No"}
      </span>
    ) : (
      "None"
    );

  const renderFleet = (fleet: ShipData["status"]["fleet"]) =>
    fleet ? (
      <span>
        {fleet.fleetType} · active: {fleet.active ? "Yes" : "No"} · system:{" "}
        {fleet.systemSymbol}
      </span>
    ) : (
      "None"
    );

  const assignmentItems: DescriptionsProps["items"] = [
    {
      key: "fleet",
      label: "Fleet",
      children: ship.status.fleetId ?? "None",
    },
    {
      key: "fleetDetails",
      label: "Fleet Details",
      span: 3,
      children: renderFleet(ship.status.fleet),
    },
    {
      key: "assignment",
      label: "Assignment",
      children: ship.status.assignmentId ?? "None",
    },
    {
      key: "assignmentDetails",
      label: "Assignment Details",
      span: 3,
      children: renderAssignment(ship.status.assignment),
    },
    {
      key: "tempFleet",
      label: "Temp Fleet",
      children: ship.status.tempFleetId ?? "None",
    },
    {
      key: "tempFleetDetails",
      label: "Temp Fleet Details",
      span: 3,
      children: renderFleet(ship.status.tempFleet),
    },
    {
      key: "tempAssignment",
      label: "Temp Assignment",
      children: ship.status.tempAssignmentId ?? "None",
    },
    {
      key: "tempAssignmentDetails",
      label: "Temp Assignment Details",
      span: 3,
      children: renderAssignment(ship.status.tempAssignment),
    },
  ];

  const summary = ship.marketTransactionSummary;
  const summaryItems: DescriptionsProps["items"] = [
    {
      key: "allExpenses",
      label: "All Expenses",
      children: <MoneyDisplay amount={summary.allExpenses ?? 0} />,
    },
    {
      key: "allIncome",
      label: "All Income",
      children: <MoneyDisplay amount={summary.allIncome ?? 0} />,
    },
    {
      key: "profit",
      label: "Profit",
      children: (
        <MoneyDisplay
          amount={(summary.allIncome ?? 0) - (summary.allExpenses ?? 0)}
        />
      ),
    },
    {
      key: "fuelExpenses",
      label: "Fuel Expenses",
      children: <MoneyDisplay amount={summary.fuelExpenses ?? 0} />,
    },
    {
      key: "fuelUnits",
      label: "Fuel Units",
      children: summary.fuelPurchaseUnits ?? 0,
    },
    {
      key: "fuelTransactions",
      label: "Fuel Transactions",
      children: summary.fuelPurchaseTransactions ?? 0,
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
      key: "allPurchaseUnits",
      label: "All Purchase Units",
      children: summary.allPurchaseUnits ?? 0,
    },
    {
      key: "allPurchaseTransactions",
      label: "All Purchase Transactions",
      children: summary.allPurchaseTransactions ?? 0,
    },
    {
      key: "chartReward",
      label: "Total Chart Reward",
      children: (
        <MoneyDisplay
          amount={
            (ship.chartTransactions.items || []).reduce(
              (a, b) => a + b.totalPrice,
              0,
            ) ?? 0
          }
        />
      ),
    },
  ];

  const purchaseItems: DescriptionsProps["items"] = ship.purchaseTransaction
    ? [
        {
          key: "id",
          label: "ID",
          children: ship.purchaseTransaction.id,
        },
        {
          key: "waypoint",
          label: "Waypoint",
          children: (
            <WaypointLink waypoint={ship.purchaseTransaction.waypointSymbol}>
              {ship.purchaseTransaction.waypointSymbol}
            </WaypointLink>
          ),
        },
        {
          key: "shipType",
          label: "Ship Type",
          children: ship.purchaseTransaction.shipType,
        },
        {
          key: "price",
          label: "Price",
          children: <MoneyDisplay amount={ship.purchaseTransaction.price} />,
        },
        {
          key: "timestamp",
          label: "Timestamp",
          children: new Date(
            ship.purchaseTransaction.timestamp,
          ).toLocaleString(),
        },
      ]
    : [];

  const repairColumns: TableProps<
    ShipData["repairTransactions"]["items"][number]
  >["columns"] = [
    { title: "ID", dataIndex: "id", key: "id" },
    {
      title: "Waypoint",
      dataIndex: "waypointSymbol",
      key: "waypointSymbol",
      render: (value: string) => (
        <WaypointLink waypoint={value}>{value}</WaypointLink>
      ),
    },
    {
      title: "Timestamp",
      dataIndex: "timestamp",
      key: "timestamp",
      render: (value: string) => new Date(value).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime(),
      defaultSortOrder: "descend",
    },
    {
      title: "Total Price",
      dataIndex: "totalPrice",
      key: "totalPrice",
      align: "right",
      render: (value: number) => <MoneyDisplay amount={value} />,
      sorter: (a, b) => a.totalPrice - b.totalPrice,
    },
  ];

  const modificationColumns: TableProps<
    ShipData["shipModificationTransactions"]["items"][number]
  >["columns"] = [
    { title: "ID", dataIndex: "id", key: "id" },
    {
      title: "Waypoint",
      dataIndex: "waypointSymbol",
      key: "waypointSymbol",
      render: (value: string) => (
        <WaypointLink waypoint={value}>{value}</WaypointLink>
      ),
    },
    {
      title: "Trade Symbol",
      dataIndex: "tradeSymbol",
      key: "tradeSymbol",
      filters: Object.values(TradeSymbol).map((symbol) => ({
        text: symbol,
        value: symbol,
      })),
      onFilter: (value, record) => record.tradeSymbol === value,
    },
    {
      title: "Timestamp",
      dataIndex: "timestamp",
      key: "timestamp",
      render: (value: string) => new Date(value).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime(),
      defaultSortOrder: "descend",
    },
    {
      title: "Total Price",
      dataIndex: "totalPrice",
      key: "totalPrice",
      align: "right",
      render: (value: number) => <MoneyDisplay amount={value} />,
      sorter: (a, b) => a.totalPrice - b.totalPrice,
    },
  ];

  const scrapTransactionsColumns: TableProps<
    ShipData["scrapTransactions"]["items"][number]
  >["columns"] = [
    { title: "ID", dataIndex: "id", key: "id" },
    {
      title: "Waypoint",
      dataIndex: "waypointSymbol",
      key: "waypointSymbol",
      render: (value: string) => (
        <WaypointLink waypoint={value}>{value}</WaypointLink>
      ),
    },
    {
      title: "Timestamp",
      dataIndex: "timestamp",
      key: "timestamp",
      render: (value: string) => new Date(value).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime(),
      defaultSortOrder: "descend",
    },
    {
      title: "Total Price",
      dataIndex: "totalPrice",
      key: "totalPrice",
      align: "right",
      render: (value: number) => <MoneyDisplay amount={value} />,
      sorter: (a, b) => a.totalPrice - b.totalPrice,
    },
  ];

  const chartColumns: TableProps<
    ShipData["chartTransactions"]["items"][number]
  >["columns"] = [
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
        (a.waypointSymbol || "").localeCompare(b.waypointSymbol || ""),
    },
    {
      title: "Waypoint Type",
      key: "waypointType",
      render: (_, record) => record.waypoint?.waypointType,
      sorter: (a, b) =>
        (a.waypoint?.waypointType ?? "").localeCompare(
          b.waypoint?.waypointType ?? "",
        ),
      filters: Object.values(WaypointType).map((t) => ({
        text: t,
        value: t,
      })),
      onFilter: (value, record) => record.waypoint?.waypointType === value,
    },
    {
      title: "Waypoint Traits",
      key: "waypointTraits",
      align: "end",
      render: (_, record) => (
        <Popover
          title={
            <Flex vertical>
              {record.waypoint?.traits.map((t) => (
                <span key={t}>{t}</span>
              ))}
            </Flex>
          }
          className="flex justify-between"
        >
          <span>
            {record.waypoint?.traits
              .map((t) =>
                t
                  .split("_")
                  .map((t) => t[0])
                  .join(""),
              )
              .join(", ")}
          </span>
          <span>{record.waypoint?.traits.length}</span>
        </Popover>
      ),
      sorter: (a, b) =>
        (a.waypoint?.traits.length ?? 0) - (b.waypoint?.traits.length ?? 0),
      filters: Object.values(WaypointTraitSymbol).map((t) => ({
        text: t,
        value: t,
      })),
      onFilter: (value, record) =>
        record.waypoint?.traits.some((t) => {
          return t == value;
        }) || false,
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
  ];

  const scrapColumns: TableProps<
    ShipData["possibleScraps"][number]
  >["columns"] = [
    {
      title: "Waypoint",
      dataIndex: "waypointSymbol",
      key: "waypointSymbol",
      render: (value: string) => (
        <WaypointLink waypoint={value}>{value}</WaypointLink>
      ),
    },
    {
      title: "Date",
      dataIndex: "date",
      key: "date",
      render: (value: string) => new Date(value).toLocaleString(),
      sorter: (a, b) => new Date(a.date).getTime() - new Date(b.date).getTime(),
      defaultSortOrder: "descend",
    },
  ];

  const constructionColumns: TableProps<
    ShipData["constructionShipments"]["items"][number]
  >["columns"] = [
    { title: "ID", dataIndex: "id", key: "id" },
    {
      title: "Trade Symbol",
      dataIndex: "tradeSymbol",
      key: "tradeSymbol",
      filters: Object.values(TradeSymbol).map((symbol) => ({
        text: symbol,
        value: symbol,
      })),
      onFilter: (value, record) => record.tradeSymbol === value,
    },
    {
      title: "Construction Site",
      dataIndex: "constructionSiteWaypoint",
      key: "constructionSiteWaypoint",
      render: (value: string) => (
        <WaypointLink waypoint={value}>{value}</WaypointLink>
      ),
    },
    {
      title: "Purchase Site",
      dataIndex: "purchaseSiteWaypoint",
      key: "purchaseSiteWaypoint",
      render: (value: string) => (
        <WaypointLink waypoint={value}>{value}</WaypointLink>
      ),
    },
    {
      title: "Units",
      dataIndex: "units",
      key: "units",
      align: "right",
      sorter: (a, b) => a.units - b.units,
    },
    {
      title: "Status",
      dataIndex: "status",
      key: "status",
      filters: Object.values(ShipmentStatus).map((status) => ({
        text: status,
        value: status,
      })),
      onFilter: (value, record) => record.status === value,
    },
    {
      title: "Expenses",
      key: "expenses",
      align: "right",
      render: (_, record) => (
        <MoneyDisplay
          amount={record.marketTransactionSummary.allExpenses ?? 0}
        />
      ),
      sorter: (a, b) =>
        (a.marketTransactionSummary.allExpenses ?? 0) -
        (b.marketTransactionSummary.allExpenses ?? 0),
    },
    {
      title: "Created At",
      dataIndex: "createdAt",
      key: "createdAt",
      render: (value: string) => new Date(value).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime(),
      defaultSortOrder: "descend",
    },
  ];

  const contractColumns: TableProps<
    ShipData["contractShipments"]["items"][number]
  >["columns"] = [
    { title: "ID", dataIndex: "id", key: "id" },
    {
      title: "Trade Symbol",
      dataIndex: "tradeSymbol",
      key: "tradeSymbol",
      filters: Object.values(TradeSymbol).map((symbol) => ({
        text: symbol,
        value: symbol,
      })),
      onFilter: (value, record) => record.tradeSymbol === value,
    },
    {
      title: "Units",
      dataIndex: "units",
      key: "units",
      align: "right",
      sorter: (a, b) => a.units - b.units,
    },
    {
      title: "Destination",
      dataIndex: "destinationSymbol",
      key: "destinationSymbol",
      render: (value: string) => (
        <WaypointLink waypoint={value}>{value}</WaypointLink>
      ),
    },
    {
      title: "Purchase",
      dataIndex: "purchaseSymbol",
      key: "purchaseSymbol",
      render: (value: string) => (
        <WaypointLink waypoint={value}>{value}</WaypointLink>
      ),
    },
    {
      title: "Status",
      dataIndex: "status",
      key: "status",
      filters: Object.values(ShipmentStatus).map((status) => ({
        text: status,
        value: status,
      })),
      onFilter: (value, record) => record.status === value,
    },
    {
      title: "Created At",
      dataIndex: "createdAt",
      key: "createdAt",
      render: (value: string) => new Date(value).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime(),
      defaultSortOrder: "descend",
    },
    {
      title: "Updated At",
      dataIndex: "updatedAt",
      key: "updatedAt",
      render: (value: string) => new Date(value).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.updatedAt).getTime() - new Date(b.updatedAt).getTime(),
    },
  ];

  const tradeRouteColumns: TableProps<
    ShipData["tradeRoutes"]["items"][number]
  >["columns"] = [
    {
      title: "Created At",
      dataIndex: "createdAt",
      key: "createdAt",
      render: (value: string) => new Date(value).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime(),
      defaultSortOrder: "descend",
    },
    {
      title: "Symbol",
      dataIndex: "symbol",
      key: "symbol",
      filters: Object.values(TradeSymbol).map((symbol) => ({
        text: symbol,
        value: symbol,
      })),
      onFilter: (value, record) => record.symbol === value,
    },
    {
      title: "Purchase WP",
      dataIndex: "PurchaseWaypointSymbol",
      key: "PurchaseWaypointSymbol",
      render: (value: string) => (
        <WaypointLink waypoint={value}>{value}</WaypointLink>
      ),
    },
    {
      title: "Sell WP",
      dataIndex: "SellWaypointSymbol",
      key: "SellWaypointSymbol",
      render: (value: string) => (
        <WaypointLink waypoint={value}>{value}</WaypointLink>
      ),
    },
    {
      title: "Status",
      dataIndex: "status",
      key: "status",
      filters: Object.values(ShipmentStatus).map((status) => ({
        text: status,
        value: status,
      })),
      onFilter: (value, record) => record.status === value,
    },
    {
      title: "Mode",
      dataIndex: "tradeMode",
      key: "tradeMode",
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
        <MoneyDisplay
          amount={record.marketTransactionSummary.allExpenses ?? 0}
        />
      ),
      sorter: (a, b) =>
        (a.marketTransactionSummary.allExpenses ?? 0) -
        (b.marketTransactionSummary.allExpenses ?? 0),
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
                    (record.purchaseMarketTradeGood?.purchasePrice || 0) *
                      record.tradeVolume || 0
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
                    ((record.purchaseMarketTradeGood?.purchasePrice || 0) *
                      record.tradeVolume || 0) -
                    (record.estimatedFuel || 0)
                  }
                />
              </Flex>
              <span className="font-bold">Summary</span>
              <Flex justify="space-between" gap={10}>
                <span>Income:</span>{" "}
                <MoneyDisplay
                  amount={record.marketTransactionSummary?.allIncome || 0}
                />
              </Flex>
              <Flex justify="space-between" gap={10}>
                <span>Expenses:</span>{" "}
                <MoneyDisplay
                  amount={record.marketTransactionSummary?.allExpenses || 0}
                />
              </Flex>
              <Flex justify="space-between" gap={10}>
                <span>Profit:</span>{" "}
                <MoneyDisplay
                  amount={
                    (record.marketTransactionSummary?.allIncome || 0) -
                    (record.marketTransactionSummary?.allExpenses || 0)
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
  ];

  const surveyColumns: TableProps<
    ShipData["surveys"]["items"][number]
  >["columns"] = [
    { title: "Signature", dataIndex: "signature", key: "signature" },
    { title: "Size", dataIndex: "size", key: "size" },
    {
      title: "Waypoint",
      dataIndex: "waypointSymbol",
      key: "waypointSymbol",
      render: (value: string) => (
        <WaypointLink waypoint={value}>{value}</WaypointLink>
      ),
    },
    {
      title: "Deposits",
      dataIndex: "deposits",
      key: "deposits",
      render: (value?: TradeSymbol[]) => (value ?? []).join(", ") || "N/A",
    },
    {
      title: "Info Before",
      dataIndex: "shipInfoBefore",
      key: "shipInfoBefore",
      align: "right",
    },
    {
      title: "Info After",
      dataIndex: "shipInfoAfter",
      key: "shipInfoAfter",
      align: "right",
    },
    {
      title: "Created At",
      dataIndex: "createdAt",
      key: "createdAt",
      render: (value: string) => new Date(value).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime(),
      defaultSortOrder: "descend",
    },
    {
      title: "Expiration",
      dataIndex: "expiration",
      key: "expiration",
      render: (value: string) => new Date(value).toLocaleString(),
    },
    {
      title: "Exhausted Since",
      dataIndex: "exhaustedSince",
      key: "exhaustedSince",
      render: (value?: string | null) =>
        value ? new Date(value).toLocaleString() : "N/A",
    },
  ];

  const extractionColumns: TableProps<
    ShipData["extractions"]["items"][number]
  >["columns"] = [
    { title: "ID", dataIndex: "id", key: "id" },
    {
      title: "Waypoint",
      dataIndex: "waypointSymbol",
      key: "waypointSymbol",
      render: (value: string) => (
        <WaypointLink waypoint={value}>{value}</WaypointLink>
      ),
    },
    { title: "Yield", dataIndex: "yieldSymbol", key: "yieldSymbol" },
    {
      title: "Units",
      dataIndex: "yieldUnits",
      key: "yieldUnits",
      align: "right",
      sorter: (a, b) => a.yieldUnits - b.yieldUnits,
    },
    {
      title: "Siphon",
      dataIndex: "siphon",
      key: "siphon",
      render: (value: boolean) => (value ? "Yes" : "No"),
    },
    {
      title: "Survey Signature",
      dataIndex: "survey_signature",
      key: "survey_signature",
      render: (value?: string | null) => value ?? "N/A",
    },
    {
      title: "Info Before",
      dataIndex: "shipInfoBefore",
      key: "shipInfoBefore",
      align: "right",
    },
    {
      title: "Info After",
      dataIndex: "shipInfoAfter",
      key: "shipInfoAfter",
      align: "right",
    },
  ];

  return (
    <>
      <Row gutter={[8, 8]}>
        <Col span={18}>
          <Flex gap={8} vertical>
            <Descriptions bordered size="small" column={4} items={infoItems} />
            <Descriptions
              bordered
              size="small"
              column={4}
              title="Assignment & Fleet"
              items={assignmentItems}
            />
          </Flex>
        </Col>
        <Col span={6}>
          <div className="flex flex-col h-full">
            <Card size="small" title="Ship Controls">
              <ShipControls ship={ship} />
            </Card>
            <Divider size="middle" />
            <Table
              size="small"
              title={() =>
                `Inventory: ${ship.cargo.units} / ${ship.cargo.capacity}`
              }
              pagination={false}
              columns={inventoryColumns}
              dataSource={ship.cargo.inventory}
              rowKey={(record) => record.symbol}
              locale={{ emptyText: <span>No cargo</span> }}
            />
            <Divider size="middle" />
            <div className="grow">
              <ShipCapabilityRadar ship={ship} />
            </div>
          </div>
        </Col>
      </Row>

      <Divider />
      <Row gutter={[8, 8]}>
        <Col span={18}>
          <Descriptions
            bordered
            size="small"
            column={4}
            title="Market Transaction Summary"
            items={summaryItems}
          />
        </Col>
        {ship.purchaseTransaction && (
          <Col span={6}>
            <>
              <Descriptions
                bordered
                size="small"
                column={2}
                title="Purchase Transaction"
                items={purchaseItems}
              />
            </>
          </Col>
        )}
      </Row>
      <Divider />
      <ShipComponents ship={ship} />

      <Divider />
      <Row gutter={[8, 8]}>
        <Col span={8}>
          <Table
            size="small"
            title={() => "Repair Transactions"}
            columns={repairColumns}
            dataSource={ship.repairTransactions.items}
            rowKey={(record) => record.id}
            pagination={{
              showSizeChanger: true,
              pageSizeOptions: ["10", "20", "50", "100"],
              defaultPageSize: 10,
              showTotal: (total, range) =>
                `${range[0]}-${range[1]} of ${total}`,
            }}
          />
        </Col>
        <Col span={8}>
          <Table
            size="small"
            title={() => "Modification Transactions"}
            columns={modificationColumns}
            dataSource={ship.shipModificationTransactions.items}
            rowKey={(record) => record.id}
            pagination={{
              showSizeChanger: true,
              pageSizeOptions: ["10", "20", "50", "100"],
              defaultPageSize: 10,
              showTotal: (total, range) =>
                `${range[0]}-${range[1]} of ${total}`,
            }}
          />
        </Col>
        <Col span={8}>
          <Table
            size="small"
            title={() => "Scrap Transactions"}
            columns={scrapTransactionsColumns}
            dataSource={ship.scrapTransactions.items}
            rowKey={(record) => record.id}
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
      <Row gutter={[8, 8]}>
        <Col span={6}>
          <Table
            size="small"
            title={() => "Possible Scraps"}
            columns={scrapColumns}
            dataSource={ship.possibleScraps}
            rowKey={(record) => record.waypointSymbol + record.date}
          />
        </Col>
        <Col span={18}>
          <Table
            size="small"
            title={() => "Chart Transactions"}
            columns={chartColumns}
            dataSource={ship.chartTransactions.items}
            rowKey={(record) =>
              record.waypointSymbol + record.timestamp + record.totalPrice
            }
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
      <Row gutter={[8, 8]}>
        <Col span={12}>
          <Table
            size="small"
            title={() => "Construction Shipments"}
            columns={constructionColumns}
            dataSource={ship.constructionShipments.items}
            rowKey={(record) => record.id}
            pagination={{
              showSizeChanger: true,
              pageSizeOptions: ["10", "20", "50", "100"],
              defaultPageSize: 10,
              showTotal: (total, range) =>
                `${range[0]}-${range[1]} of ${total}`,
            }}
          />
        </Col>
        <Col span={12}>
          <Table
            size="small"
            title={() => "Contract Shipments"}
            columns={contractColumns}
            dataSource={ship.contractShipments.items}
            rowKey={(record) => record.id}
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
      <Table
        size="small"
        title={() => "Trade Routes"}
        columns={tradeRouteColumns}
        dataSource={ship.tradeRoutes.items}
        rowKey={(record) => record.id}
        pagination={{
          showSizeChanger: true,
          pageSizeOptions: ["10", "20", "50", "100"],
          defaultPageSize: 10,
          showTotal: (total, range) => `${range[0]}-${range[1]} of ${total}`,
        }}
      />

      <Divider />
      <Row gutter={[8, 8]}>
        <Col span={12}>
          <Table
            size="small"
            title={() => "Surveys"}
            columns={surveyColumns}
            dataSource={ship.surveys.items}
            rowKey={(record) => record.signature}
            pagination={{
              showSizeChanger: true,
              pageSizeOptions: ["10", "20", "50", "100"],
              defaultPageSize: 10,
              showTotal: (total, range) =>
                `${range[0]}-${range[1]} of ${total}`,
            }}
          />
        </Col>
        <Col span={12}>
          <Table
            size="small"
            title={() => "Extractions"}
            columns={extractionColumns}
            dataSource={ship.extractions.items}
            rowKey={(record) => record.id}
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
    </>
  );
}

function Conditions({ conditions }: { conditions: ShipData["conditions"] }) {
  return (
    <Flex gap={8} vertical align="center" justify="center">
      <Space>
        Engine:
        <Progress
          type="circle"
          percent={conditions.engine.condition * 100}
          size={20}
          format={(value) => `Condition: ${value}%`}
        />
        <Progress
          type="circle"
          percent={conditions.engine.integrity * 100}
          size={20}
          format={(value) => `Integrity: ${value}%`}
        />
      </Space>
      <Space>
        Frame:
        <Progress
          type="circle"
          percent={conditions.frame.condition * 100}
          size={20}
          format={(value) => `Condition: ${value}%`}
        />
        <Progress
          type="circle"
          percent={conditions.frame.integrity * 100}
          size={20}
          format={(value) => `Integrity: ${value}%`}
        />
      </Space>
      <Space>
        Reactor:
        <Progress
          type="circle"
          percent={conditions.reactor.condition * 100}
          size={20}
          format={(value) => `Condition: ${value}%`}
        />
        <Progress
          type="circle"
          percent={conditions.reactor.integrity * 100}
          size={20}
          format={(value) => `Integrity: ${value}%`}
        />
      </Space>
    </Flex>
  );
}

function ShipNavProgress({
  departureTime,
  arrival,
}: {
  departureTime: string;
  arrival: string;
}) {
  const [percent, setPercent] = useState(0);

  useEffect(() => {
    const interval = setInterval(() => {
      setPercent(
        Math.round(
          ((new Date().getTime() - new Date(departureTime).getTime()) /
            (new Date(arrival).getTime() - new Date(departureTime).getTime())) *
            10000,
        ) / 100,
      );
    }, 100);

    return () => clearInterval(interval);
  }, [arrival, departureTime]);

  return (
    <Progress
      percent={percent}
      size="small"
      format={(value) => `${value?.toFixed(2)}%`}
    />
  );
}

export default Ship;
export type { ShipData };
