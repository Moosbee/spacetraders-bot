import { useQuery } from "@apollo/client/react";
import {
  Button,
  Divider,
  Flex,
  Popover,
  Result,
  Space,
  Table,
  TableProps,
} from "antd";
import { Link } from "react-router-dom";
import AssignmentsPopover from "../features/AssignmentsPopover/AssignmentsPopover";
import {
  configSummary,
  configToEntries,
  formatConfigValue,
} from "../features/FleetConfig/ConfigRendering";
import PageTitle from "../features/PageTitle";
import { FleetType, GetFleetsQuery } from "../gql/graphql";
import { GET_FLEETS } from "../graphql/queries";
type FleetRecord = GetFleetsQuery["fleets"]["items"][number];

function Fleets() {
  const { loading, error, data, refetch } = useQuery(GET_FLEETS);

  const columns: TableProps<FleetRecord>["columns"] = [
    {
      title: "ID",
      dataIndex: "id",
      key: "id",
      defaultSortOrder: "ascend",
      render: (id: number) => <Link to={`/fleets/${id}`}>{id}</Link>,
      sorter: (a, b) => a.id - b.id,
    },
    {
      title: "System",
      dataIndex: "systemSymbol",
      key: "systemSymbol",
      render: (systemSymbol: string) => (
        <Link to={`/system/${systemSymbol}`}>{systemSymbol}</Link>
      ),
      sorter: (a, b) => a.systemSymbol.localeCompare(b.systemSymbol),
    },
    {
      title: "Type",
      dataIndex: "fleetType",
      key: "fleetType",
      sorter: (a, b) => a.fleetType.localeCompare(b.fleetType),
      filters: Object.values(FleetType).map((type) => ({
        text: type,
        value: type,
      })),
      onFilter: (value, record) => record.fleetType === value,
    },
    {
      title: "Active",
      dataIndex: "active",
      key: "active",
      render: (active: boolean) => (active ? "Active" : "Inactive"),
      sorter: (a, b) => Number(a.active) - Number(b.active),
    },
    {
      title: "Assignments",
      key: "assignments",
      render: (_: unknown, record) => {
        const assignments = record.assignments.items;
        const assigned = assignments.filter((a) => a.ship.length > 0).length;
        return (
          <Popover title={<AssignmentsPopover assignments={assignments} />}>
            <span>
              {assigned}/{assignments.length}
            </span>
          </Popover>
        );
      },
      sorter: (a, b) => a.assignments.items.length - b.assignments.items.length,
    },
    {
      title: "Config",
      key: "config",
      render: (_: unknown, record) => {
        const config = record.config;
        return (
          <Popover
            title={config.__typename}
            content={
              <Flex vertical gap={2}>
                {configToEntries(config).map(([key, value]) => (
                  <Flex key={key} justify="space-between" gap={16}>
                    <span>{key}</span>
                    <span>{formatConfigValue(value)}</span>
                  </Flex>
                ))}
              </Flex>
            }
          >
            <span>{configSummary(config)}</span>
          </Popover>
        );
      },
    },
  ];

  if (error) {
    return (
      <Result
        status="error"
        title="Fleets Error"
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
      <PageTitle title={`Fleets`} />
      <Space>
        <h1 className="scroll-m-20 text-center text-3xl font-bold tracking-tight text-balance">
          Fleets
        </h1>
        <Button onClick={() => refetch()}>Refresh</Button>
      </Space>
      <Divider />
      <Table
        loading={loading}
        dataSource={data?.fleets.items ?? []}
        columns={columns}
        rowKey={(record) => record.id}
      />
    </div>
  );
}

export default Fleets;
