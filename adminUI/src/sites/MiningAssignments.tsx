import { useQuery } from "@apollo/client/react";
import {
  Button,
  Card,
  Descriptions,
  Divider,
  List,
  Space,
  Spin,
  Table,
  TableProps,
} from "antd";
import { Link } from "react-router-dom";
import PageTitle from "../features/PageTitle";
import { GetMiningAssignmentsQuery } from "../gql/graphql";
import { GET_MINING_ASSIGNMENTS } from "../graphql/queries";

type GQLFleet = GetMiningAssignmentsQuery["fleets"]["items"][number];

function miningConfigOf(fleet: GQLFleet) {
  return fleet.config.__typename === "MiningConfig" ? fleet.config : undefined;
}

export default function MiningAssignments() {
  const { loading, error, data, dataState, refetch } = useQuery(
    GET_MINING_ASSIGNMENTS,
  );

  if (dataState !== "complete") return <p>Loading...</p>;
  if (error) return <p>Error: {error.message}</p>;

  const { miningManager } = data;
  const assignments = miningManager.getAssignments;
  const fleets = data.fleets.items;

  const fleetColumns: TableProps<GQLFleet>["columns"] = [
    {
      title: "ID",
      dataIndex: "id",
      key: "id",
    },
    {
      title: "System",
      dataIndex: "systemSymbol",
      key: "systemSymbol",
      render: (symbol: string) => (
        <Link to={`/system/${symbol}`}>{symbol}</Link>
      ),
    },
    {
      title: "Active",
      dataIndex: "active",
      key: "active",
      render: (value: boolean) => (value ? "Yes" : "No"),
    },
    {
      title: "Assignments",
      key: "assignments",
      render: (_, fleet) => fleet.assignments.items.length,
    },
    {
      title: "Ships",
      key: "ships",
      render: (_, fleet) =>
        fleet.assignments.items.reduce((sum, a) => sum + a.ship.length, 0),
    },
    {
      title: "Miners/WP",
      key: "minersPerWaypoint",
      render: (_, fleet) => miningConfigOf(fleet)?.minersPerWaypoint ?? "-",
    },
    {
      title: "Siphoners/WP",
      key: "siphonersPerWaypoint",
      render: (_, fleet) => miningConfigOf(fleet)?.siphonersPerWaypoint ?? "-",
    },
    {
      title: "Surveyors/WP",
      key: "surveyersPerWaypoint",
      render: (_, fleet) => miningConfigOf(fleet)?.surveyersPerWaypoint ?? "-",
    },
    {
      title: "Transporters/WP",
      key: "miningTransportersPerWaypoint",
      render: (_, fleet) =>
        miningConfigOf(fleet)?.miningTransportersPerWaypoint ?? "-",
    },
    {
      title: "Mining Waypoints",
      key: "miningWaypoints",
      render: (_, fleet) => miningConfigOf(fleet)?.miningWaypoints ?? "-",
    },
    {
      title: "Syphon Waypoints",
      key: "syphonWaypoints",
      render: (_, fleet) => miningConfigOf(fleet)?.syphonWaypoints ?? "-",
    },
    {
      title: "Min Mining Cargo",
      key: "minMiningCargoSpace",
      render: (_, fleet) => miningConfigOf(fleet)?.minMiningCargoSpace ?? "-",
    },
    {
      title: "Min Siphon Cargo",
      key: "minSiphonCargoSpace",
      render: (_, fleet) => miningConfigOf(fleet)?.minSiphonCargoSpace ?? "-",
    },
    {
      title: "Min Transporter Cargo",
      key: "minTransporterCargoSpace",
      render: (_, fleet) =>
        miningConfigOf(fleet)?.minTransporterCargoSpace ?? "-",
    },
    {
      title: "Ignore Eng. Asteroids",
      key: "ignoreEngineeredAsteroids",
      render: (_, fleet) =>
        miningConfigOf(fleet)?.ignoreEngineeredAsteroids ? "Yes" : "No",
    },
    {
      title: "Stop Unstable",
      key: "stopAllUnstable",
      render: (_, fleet) =>
        miningConfigOf(fleet)?.stopAllUnstable ? "Yes" : "No",
    },
    {
      title: "Unstable Timeout",
      key: "unstableSinceTimeout",
      render: (_, fleet) => miningConfigOf(fleet)?.unstableSinceTimeout ?? "-",
    },
    {
      title: "Eject List",
      key: "miningEjectList",
      render: (_, fleet) =>
        miningConfigOf(fleet)?.miningEjectList.join(", ") ?? "-",
    },
    {
      title: "Prefer List",
      key: "miningPreferList",
      render: (_, fleet) =>
        miningConfigOf(fleet)?.miningPreferList.join(", ") ?? "-",
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="Mining Assignments" />
      <Space>
        <h1>
          Mining Assignments{" "}
          {assignments.reduce((sum, a) => sum + a.assignedShips.length, 0)}
        </h1>
        <Button
          onClick={() => {
            refetch();
          }}
        >
          Refresh
        </Button>
      </Space>

      <Descriptions
        size="small"
        bordered
        column={3}
        items={[
          {
            label: "Busy",
            key: "busy",
            children: <Spin spinning={miningManager.busy} size="small" />,
          },
          {
            label: "Channel State",
            key: "channelState",
            children: (
              <span
                style={{
                  color:
                    miningManager.channelState.state === "CLOSED"
                      ? "red"
                      : "green",
                }}
              >
                {miningManager.channelState.state}
              </span>
            ),
          },
          {
            label: "Used Capacity",
            key: "usedCapacity",
            children: miningManager.channelState.usedCapacity,
          },
        ]}
      />

      <List
        loading={loading}
        grid={{ gutter: 16, column: 4 }}
        dataSource={assignments}
        renderItem={(item) => (
          <List.Item>
            <Card title={item.waypointSymbol}>
              <List
                dataSource={item.assignedShips}
                renderItem={(ship) => (
                  <List.Item>
                    <Link to={`/ships/${ship.shipSymbol}`}>
                      {ship.shipSymbol}
                    </Link>
                    : {ship.level}
                  </List.Item>
                )}
              />
            </Card>
          </List.Item>
        )}
      />

      <Divider />

      <h2>Mining Fleets</h2>
      <Table
        size="small"
        loading={loading}
        rowKey="id"
        columns={fleetColumns}
        dataSource={fleets}
        scroll={{ x: "max-content" }}
        pagination={false}
      />
    </div>
  );
}
