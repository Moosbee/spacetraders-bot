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
import FleetConfigDetails from "../features/FleetConfig/FleetConfig";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import WaypointLink from "../features/WaypointLink";
import { GetFleetQuery } from "../gql/graphql";
import { GET_FLEET } from "../graphql/queries";

type FleetData = GetFleetQuery["fleet"];
type Assignment = FleetData["assignments"]["items"][number];
type FleetShip = FleetData["allShips"][number];

const assignmentColumns: TableProps<Assignment>["columns"] = [
  {
    title: "ID",
    dataIndex: "id",
    key: "id",
    defaultSortOrder: "ascend",
    sorter: (a, b) => a.id - b.id,
  },
  {
    title: "Ships",
    key: "ships",
    render: (_, record) => (
      <Flex wrap gap={4}>
        {record.ship.map((s) => (
          <Link key={s.symbol} to={`/ships/${s.symbol}`}>
            {s.symbol}
          </Link>
        ))}
      </Flex>
    ),
  },
  {
    title: "Priority",
    dataIndex: "priority",
    key: "priority",
    sorter: (a, b) => a.priority - b.priority,
  },
  {
    title: "Range Min",
    dataIndex: "rangeMin",
    key: "rangeMin",
    align: "right",
    sorter: (a, b) => a.rangeMin - b.rangeMin,
  },
  {
    title: "Cargo Min",
    dataIndex: "cargoMin",
    key: "cargoMin",
    align: "right",
    sorter: (a, b) => a.cargoMin - b.cargoMin,
  },
  {
    title: "Max Purchase",
    dataIndex: "maxPurchasePrice",
    key: "maxPurchasePrice",
    align: "right",
    render: (value: number) => <MoneyDisplay amount={value} />,
    sorter: (a, b) => a.maxPurchasePrice - b.maxPurchasePrice,
  },
  {
    title: "Credits Threshold",
    dataIndex: "creditsThreshold",
    key: "creditsThreshold",
    align: "right",
    render: (value: number) => <MoneyDisplay amount={value} />,
    sorter: (a, b) => a.creditsThreshold - b.creditsThreshold,
  },
  {
    title: "Flags",
    key: "flags",
    render: (_, record) => (
      <Flex gap={4}>
        {record.extractor && "Extractor"}
        {record.siphon && "Siphon"}
        {record.survey && "Survey"}
        {record.warpDrive && "Warp Drive"}
      </Flex>
    ),
  },
  {
    title: "State",
    dataIndex: "disabled",
    key: "disabled",
    render: (disabled: boolean) => (disabled ? "Disabled" : "Active"),
    sorter: (a, b) => Number(a.disabled) - Number(b.disabled),
  },
];

const shipColumns: TableProps<FleetShip>["columns"] = [
  {
    title: "Symbol",
    dataIndex: "symbol",
    key: "symbol",
    render: (symbol: string) => <Link to={`/ships/${symbol}`}>{symbol}</Link>,
    sorter: (a, b) => a.symbol.localeCompare(b.symbol),
  },
  {
    title: "Status",
    key: "status",
    render: (_, record) => record.status.status.__typename,
  },
  {
    title: "Waypoint",
    dataIndex: "waypointSymbol",
    key: "waypointSymbol",
    render: (_, record) => (
      <WaypointLink waypoint={record.nav.waypointSymbol}>
        {record.nav.waypointSymbol}
      </WaypointLink>
    ),
    sorter: (a, b) => a.nav.waypointSymbol.localeCompare(b.nav.waypointSymbol),
  },
  {
    title: "Nav Status",
    key: "navStatus",
    render: (_, record) => record.nav.status,
    sorter: (a, b) => a.nav.status.localeCompare(b.nav.status),
  },
  {
    title: "Fuel",
    key: "fuel",
    render: (_, record) => record.fuel.capacity,
    sorter: (a, b) => a.fuel.capacity - b.fuel.capacity,
    align: "right",
  },
  {
    title: "Cargo",
    key: "cargo",
    render: (_, record) => record.cargo.capacity,
    sorter: (a, b) => a.cargo.capacity - b.cargo.capacity,
    align: "right",
  },
  {
    title: "Engine Speed",
    key: "engineSpeed",
    render: (_, record) => record.engineSpeed,
    sorter: (a, b) => a.engineSpeed - b.engineSpeed,
    align: "right",
  },
];

function Fleet() {
  const { fleetID } = useParams();

  const { loading, error, data, refetch } = useQuery(GET_FLEET, {
    variables: { fleetID: Number(fleetID) },
  });

  const fleet = data?.fleet;

  if (error) {
    return (
      <div style={{ padding: "24px 24px" }}>
        <PageTitle title={`Fleet ${fleetID}`} />
        <Result
          status="error"
          title="Fleet Error"
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

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle
        title={`Fleet ${fleet?.fleetType ? fleet?.fleetType + "_" : ""}${fleetID}`}
      />
      <Spin spinning={loading}>
        <Space wrap>
          <h1 className="scroll-m-20 text-center text-3xl font-bold tracking-tight text-balance">
            Fleet {fleetID}
          </h1>
          <Button onClick={() => refetch()} loading={loading}>
            Refresh
          </Button>
        </Space>
        <Divider />
        {fleet ? (
          <FleetDetails fleet={fleet} />
        ) : (
          !loading && (
            <Result
              status="404"
              title="Fleet not found"
              subTitle={`No fleet with id ${fleetID}`}
            />
          )
        )}
      </Spin>
    </div>
  );
}

function FleetDetails({ fleet }: { fleet: FleetData }) {
  const infoItems: DescriptionsProps["items"] = [
    { key: "id", label: "ID", children: fleet.id },
    {
      key: "system",
      label: "System",
      children: (
        <Link to={`/system/${fleet.systemSymbol}`}>{fleet.systemSymbol}</Link>
      ),
    },
    { key: "type", label: "Type", children: fleet.fleetType },
    {
      key: "active",
      label: "Active",
      children: fleet.active ? "Active" : "Inactive",
    },
  ];

  return (
    <>
      <Descriptions bordered size="small" column={4} items={infoItems} />
      <Divider />

      <FleetConfigDetails config={fleet.config} />
      <Divider />
      <Row gutter={[8, 8]}>
        <Col span={15}>
          <Table
            title={() => `Assignments (${fleet.assignments.items.length})`}
            size="small"
            dataSource={fleet.assignments.items}
            columns={assignmentColumns}
            rowKey={(record) => record.id}
            pagination={false}
          />
        </Col>
        <Col span={9}>
          <Table
            title={() => `Ships (${fleet.allShips.length})`}
            size="small"
            dataSource={fleet.allShips}
            columns={shipColumns}
            rowKey={(record) => record.symbol}
            pagination={false}
          />
        </Col>
      </Row>
    </>
  );
}

export default Fleet;
