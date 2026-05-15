import { useQuery } from "@apollo/client/react";
import {
  Alert,
  Card,
  Descriptions,
  Flex,
  List,
  Progress,
  Space,
  Table,
  Tag,
  Typography,
} from "antd";
import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import PageTitle from "../features/PageTitle";
import RoleRenderer from "../features/RoleRenderer/RoleRenderer";
import ShipControl from "../features/ShipControl/ShipControl";
import Timer from "../features/Timer/Timer";
import WaypointLink from "../features/WaypointLink";
import { GetShipEventsQuery } from "../gql/graphql";
import { GET_SHIP_EVENTS, SHIP_EVENTS_SUBSCRIPTION } from "../graphql/queries";
import { useAppSelector } from "../redux/hooks";
import { selectShip } from "../redux/slices/shipSlice";

type GQLShipEvent = NonNullable<
  GetShipEventsQuery["ship"]
>["shipEvents"]["items"][number];

function Ship() {
  const { shipID } = useParams();
  const ship = useAppSelector((state) => selectShip(state, shipID));

  if (!ship) return <div>Ship not found</div>;

  const {
    loading: shipEventsLoading,
    error: shipEventsError,
    data: shipEventsData,
    subscribeToMore,
  } = useQuery(GET_SHIP_EVENTS, {
    variables: { shipSymbol: ship.symbol },
  });

  useEffect(() => {
    const unsubscribe = subscribeToMore({
      document: SHIP_EVENTS_SUBSCRIPTION,
      variables: { shipSymbol: ship.symbol },
      updateQuery: (previous, { subscriptionData }) => {
        const nextEvent = subscriptionData.data?.shipEvents;
        const currentShip = previous.ship;
        const currentShipEvents = currentShip?.shipEvents;
        const currentItems = currentShipEvents?.items;

        if (!nextEvent || !currentShip || !currentShipEvents || !currentItems) {
          return previous as GetShipEventsQuery;
        }

        if (currentItems.some((event) => event?.id === nextEvent.id)) {
          return previous as GetShipEventsQuery;
        }

        return {
          ...previous,
          ship: {
            ...currentShip,
            shipEvents: {
              ...currentShipEvents,
              totalCount:
                (currentShipEvents.totalCount ?? currentItems.length) + 1,
              items: [...currentItems, nextEvent],
            },
          },
        } as GetShipEventsQuery;
      },
    });

    return unsubscribe;
  }, [ship.symbol, subscribeToMore]);

  const shipEvents = (shipEventsData?.ship?.shipEvents.items ?? [])
    .slice()
    .reverse();

  // console.log(ship);

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title={`Ship ${ship.symbol}`} />
      <Space>
        <h2>Ship {ship.symbol}</h2>
      </Space>
      <Flex gap={8} justify="space-between" align="center">
        <Descriptions
          bordered
          size="small"
          column={2}
          // layout="vertical"
          items={[
            {
              label: "Symbol",
              key: "symbol",
              children: ship.symbol,
            },
            {
              label: "Display Name",
              key: "display_name",
              children: ship.display_name,
            },
            {
              label: "Role",
              key: "role",
              children: ship.status.type,
            },
            {
              label: "Status",
              key: "status",
              children: <RoleRenderer status={ship.status} />,
            },
            {
              label: "Registration Role",
              key: "registration_role",
              children: ship.registration_role,
            },
            {
              label: "Active",
              key: "active",
              children: ship.active ? "Yes" : "No",
            },

            ...(ship.cooldown_expiration
              ? [
                  {
                    label: "Cooldown",
                    key: "cooldown",
                    span: 2,
                    children: (
                      <span>
                        <Timer time={ship.cooldown_expiration} />
                      </span>
                    ),
                  },
                ]
              : []),
            {
              label: "Fuel",
              key: "fuel",
              children: `${ship.fuel.current} / ${ship.fuel.capacity}`,
            },
            {
              label: "Conditions",
              key: "conditions",
              children: (
                <Flex gap={8} vertical align="center" justify="center">
                  <Space>
                    Engine:
                    <Progress
                      type="circle"
                      percent={ship.conditions.engine.condition * 100}
                      size={20}
                      format={(value) => `Condition: ${value}%`}
                    />
                    <Progress
                      type="circle"
                      percent={ship.conditions.engine.integrity * 100}
                      format={(value) => `Integrity: ${value}%`}
                      size={20}
                    />
                  </Space>
                  <Space>
                    Frame:
                    <Progress
                      type="circle"
                      percent={ship.conditions.frame.condition * 100}
                      format={(value) => `Condition: ${value}%`}
                      size={20}
                    />
                    <Progress
                      type="circle"
                      percent={ship.conditions.frame.integrity * 100}
                      format={(value) => `Integrity: ${value}%`}
                      size={20}
                    />
                  </Space>
                  <Space>
                    Reactor:
                    <Progress
                      type="circle"
                      percent={ship.conditions.reactor.condition * 100}
                      format={(value) => `Condition: ${value}%`}
                      size={20}
                    />
                    <Progress
                      type="circle"
                      percent={ship.conditions.reactor.integrity * 100}
                      format={(value) => `Integrity: ${value}%`}
                      size={20}
                    />
                  </Space>
                </Flex>
              ),
            },
            {
              label: "System Symbol",
              key: "system_symbol",
              children: (
                <Link to={`/system/${ship.nav.system_symbol}`}>
                  {ship.nav.system_symbol}
                </Link>
              ),
            },
            {
              label: "Waypoint Symbol",
              key: "waypoint_symbol",
              children: (
                <WaypointLink waypoint={ship.nav.waypoint_symbol}>
                  {ship.nav.waypoint_symbol}
                </WaypointLink>
              ),
            },
            {
              label: "Flight Mode",
              key: "flight_mode",
              children: ship.nav.flight_mode,
            },
            {
              label: "Engine Speed",
              key: "engine_speed",
              children: ship.engine_speed,
            },
            {
              label: "Status",
              key: "status",
              span: 2,
              children: (
                <span>
                  {ship.nav.status}
                  {ship.nav.status === "IN_TRANSIT" && (
                    <>
                      (<Timer time={ship.nav.route.arrival} />)
                      <ShipNavProgress {...ship.nav.route} />
                      <br />
                      <span>
                        {ship.nav.route.origin_symbol} -{">"}{" "}
                        {ship.nav.route.destination_symbol}
                      </span>
                    </>
                  )}
                </span>
              ),
            },
            ...(ship.nav.auto_pilot
              ? [
                  {
                    label: "Auto Pilot",
                    key: "auto_pilot",
                    span: 2,
                    children: (
                      <span>
                        {ship.nav.auto_pilot.origin_system_symbol ==
                        ship.nav.auto_pilot.destination_system_symbol
                          ? ship.nav.auto_pilot.origin_symbol.replace(
                              ship.nav.auto_pilot.origin_system_symbol + "-",
                              "",
                            )
                          : ship.nav.auto_pilot.origin_symbol}{" "}
                        -{">"}{" "}
                        {ship.nav.auto_pilot.origin_system_symbol ===
                        ship.nav.auto_pilot.destination_system_symbol
                          ? ship.nav.auto_pilot.destination_symbol.replace(
                              ship.nav.auto_pilot.destination_system_symbol +
                                "-",
                              "",
                            )
                          : ship.nav.auto_pilot.destination_symbol}
                        <br />
                        <ShipNavProgress
                          departure_time={ship.nav.auto_pilot.departure_time}
                          arrival={ship.nav.auto_pilot.arrival}
                        />
                        <br />(
                        <Timer time={ship.nav.auto_pilot.arrival} />)
                        <br />
                        <List
                          size="small"
                          bordered
                          dataSource={ship.nav.auto_pilot.route.connections}
                          renderItem={(item) => (
                            <List.Item>
                              <>
                                {item.Navigate && (
                                  <>
                                    <Typography.Text
                                      mark={
                                        ship.nav.waypoint_symbol ===
                                        item.Navigate.end_symbol
                                      }
                                    >
                                      {item.Navigate.nav_mode}{" "}
                                      {item.Navigate.start_symbol} -{">"}{" "}
                                      {item.Navigate.end_symbol} (
                                      {item.Navigate.travel_time}s)
                                    </Typography.Text>
                                  </>
                                )}
                                {item.JumpGate && (
                                  <>
                                    <Typography.Text
                                      mark={
                                        ship.nav.waypoint_symbol ===
                                        item.JumpGate.end_symbol
                                      }
                                    >
                                      {item.JumpGate.start_symbol} -{">"}{" "}
                                      {item.JumpGate.end_symbol} (
                                      {item.JumpGate.distance})
                                    </Typography.Text>
                                  </>
                                )}
                              </>
                            </List.Item>
                          )}
                        />
                      </span>
                    ),
                  },
                ]
              : []),
          ]}
        />
        <ShipControl ship={ship} />

        <Table
          size="small"
          title={() =>
            `Inventory: ${ship.cargo.units} / ${ship.cargo.capacity}`
          }
          pagination={false}
          columns={[
            {
              title: "Type",
              dataIndex: "0",
              key: "type",
            },
            {
              title: "Amount",
              dataIndex: "1",
              key: "amount",
              align: "right",
            },
          ]}
          dataSource={Object.entries(ship.cargo.inventory)}
          rowKey={(type) => type[0]}
        />
        <Table
          size="small"
          title={() => "Mounts"}
          pagination={false}
          columns={[
            {
              title: "Type",
              dataIndex: "0",
              key: "type",
            },
          ]}
          dataSource={ship.mounts.mounts.map((m, i) => [m, i])}
          rowKey={(type) => type[1]}
        />
        <Table
          size="small"
          title={() => "Modules"}
          pagination={false}
          columns={[
            {
              title: "Type",
              dataIndex: "0",
              key: "type",
            },
          ]}
          dataSource={ship.modules.modules.map((m, i) => [m, i])}
          rowKey={(type) => type[1]}
        />
      </Flex>
      <Card
        size="small"
        title={`Ship Events (${shipEventsData?.ship?.shipEvents.totalCount ?? 0})`}
        style={{ marginTop: 16 }}
      >
        {shipEventsError ? (
          <Alert
            type="error"
            showIcon
            message="Could not load ship events"
            description={shipEventsError.message}
          />
        ) : (
          <Table<GQLShipEvent>
            size="small"
            loading={shipEventsLoading}
            rowKey={(event) => event.id}
            pagination={{ pageSize: 10, showSizeChanger: false }}
            dataSource={shipEvents}
            locale={{ emptyText: "No ship events yet" }}
            columns={[
              {
                title: "Created",
                dataIndex: "createdAt",
                key: "createdAt",
                width: 220,
                defaultSortOrder: "descend",
                sorter: (a, b) => a.createdAt.localeCompare(b.createdAt),
                render: (createdAt: string) => (
                  <span>{new Date(createdAt).toLocaleString()}</span>
                ),
              },
              {
                title: "Event",
                key: "event",
                width: 260,
                render: (_value, record) => (
                  <Space wrap size={[4, 4]}>
                    <Tag color="blue">{record.eventKind}</Tag>
                    <Tag>{record.eventName}</Tag>
                    <Tag color="green">{record.eventPhase}</Tag>
                  </Space>
                ),
              },
              {
                title: "Details",
                key: "payload",
                render: (_value, record) => renderShipEventPayload(record),
              },
              {
                title: "States",
                key: "states",
                width: 140,
                render: (_value, record) => (
                  <span>
                    {record.beforeShipStateId} -{">"} {record.afterShipStateId}
                  </span>
                ),
              },
            ]}
          />
        )}
      </Card>
    </div>
  );
}

function renderShipEventPayload(event: GQLShipEvent) {
  switch (event.payload.__typename) {
    case "JumpConnectionCompletedEvent":
      return (
        <span>
          Jumped {event.payload.from} -{">"} {event.payload.to} (
          {event.payload.jumpDistance})
        </span>
      );
    case "WarpConnectionCompletedEvent":
      return (
        <span>
          Warped {event.payload.from} -{">"} {event.payload.to} in{" "}
          {event.payload.navMode} ({Math.round(event.payload.travelDistance)}u,{" "}
          {Math.round(event.payload.travelTime)}s)
        </span>
      );
    case "NavigateConnectionCompletedEvent":
      return (
        <span>
          Navigated {event.payload.from} -{">"} {event.payload.to} in{" "}
          {event.payload.navMode} ({Math.round(event.payload.travelDistance)}u,{" "}
          {Math.round(event.payload.travelTime)}s)
        </span>
      );
    case "CargoTradeCompletedEvent":
      return (
        <span>
          {event.payload.transactionType} {event.payload.units}{" "}
          {event.payload.tradeSymbol} at {event.payload.waypointSymbol}
          {event.payload.contractId
            ? ` contract ${event.payload.contractId}`
            : ""}
        </span>
      );
    case "MiningExtractionCompletedEvent":
      return (
        <span>
          {event.payload.siphon ? "Siphoned" : "Extracted"}{" "}
          {event.payload.yieldUnits} {event.payload.yieldSymbol} at{" "}
          {event.payload.waypointSymbol}
          {event.payload.surveySignature
            ? ` via ${event.payload.surveySignature}`
            : ""}
        </span>
      );
    case "MiningSurveyCreatedEvent":
      return (
        <span>
          Created {event.payload.surveysCreated} survey
          {event.payload.surveysCreated === 1 ? "" : "s"} at{" "}
          {event.payload.waypointSymbol}
          {event.payload.surveySignatures.length > 0
            ? ` (${event.payload.surveySignatures.join(", ")})`
            : ""}
        </span>
      );
    default:
      return (
        <Typography.Text type="secondary">Unknown payload</Typography.Text>
      );
  }
}

function ShipNavProgress({
  departure_time,
  arrival,
}: {
  departure_time: string;
  arrival: string;
}) {
  const [percent, setPercent] = useState(0);

  useEffect(() => {
    const interval = setInterval(() => {
      setPercent(
        Math.round(
          ((new Date().getTime() - new Date(departure_time).getTime()) /
            (new Date(arrival).getTime() -
              new Date(departure_time).getTime())) *
            10000,
        ) / 100,
      );
    }, 100);

    return () => clearInterval(interval);
  }, [arrival, departure_time]);

  return (
    <Progress
      percent={percent}
      size="small"
      format={(value) => `${value?.toFixed(2)}%`}
    />
  );
}

export default Ship;
