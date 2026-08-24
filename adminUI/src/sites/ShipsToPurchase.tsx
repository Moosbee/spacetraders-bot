import { useQuery } from "@apollo/client/react";
import { Button, Result, Space, Spin, Table, TableProps } from "antd";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import { FleetType, GetOpenAssignmentsQuery } from "../gql/graphql";
import { GET_OPEN_ASSIGNMENTS } from "../graphql/queries";

type Assignment = GetOpenAssignmentsQuery["shipAssignments"]["items"][number];

const boolFilters = [
  { text: "Yes", value: true },
  { text: "No", value: false },
];

const renderBool = (value: boolean) => (value ? "Yes" : "No");

export default function ShipsToPurchase() {
  const { loading, error, data, dataState, refetch } = useQuery(
    GET_OPEN_ASSIGNMENTS,
    {},
  );

  if (error) {
    return (
      <div style={{ width: "100%", height: "100%", position: "relative" }}>
        <PageTitle title={`Systems Map`} />
        <Result
          status="error"
          title="System Map Error"
          subTitle={`Error: ${error.message}`}
          extra={[
            <Button key="tryAgain" type="primary" onClick={() => refetch()}>
              Try Again
            </Button>,
          ]}
        ></Result>
      </div>
    );
  }

  const items = data?.shipAssignments.items || [];

  const systemFilters = [
    ...new Set(
      items.map((a) => a.fleet?.systemSymbol).filter((s): s is string => !!s),
    ),
  ]
    .toSorted()
    .map((symbol) => ({ text: symbol, value: symbol }));

  const columns: TableProps<Assignment>["columns"] = [
    {
      title: "ID",
      dataIndex: "id",
      key: "id",
      sorter: (a, b) => a.id - b.id,
      defaultSortOrder: "descend",
    },
    {
      title: "Fleet ID",
      key: "fleetId",
      render: (_, record) => record.fleet?.id ?? "N/A",
      sorter: (a, b) => (a.fleet?.id ?? 0) - (b.fleet?.id ?? 0),
    },
    {
      title: "Fleet Type",
      key: "fleetType",
      render: (_, record) => record.fleet?.fleetType ?? "N/A",
      filters: Object.values(FleetType).map((type) => ({
        text: type,
        value: type,
      })),
      onFilter: (value, record) => record.fleet?.fleetType === value,
      sorter: (a, b) =>
        (a.fleet?.fleetType ?? "").localeCompare(b.fleet?.fleetType ?? ""),
    },
    {
      title: "Active",
      key: "active",
      render: (_, record) => renderBool(record.fleet?.active ?? false),
      filters: boolFilters,
      onFilter: (value, record) => (record.fleet?.active ?? false) === value,
      sorter: (a, b) =>
        Number(a.fleet?.active ?? false) - Number(b.fleet?.active ?? false),
    },
    {
      title: "System",
      key: "system",
      render: (_, record) => record.fleet?.systemSymbol ?? "N/A",
      filters: systemFilters,
      onFilter: (value, record) => record.fleet?.systemSymbol === value,
      sorter: (a, b) =>
        (a.fleet?.systemSymbol ?? "").localeCompare(
          b.fleet?.systemSymbol ?? "",
        ),
    },
    {
      title: "Priority",
      dataIndex: "priority",
      key: "priority",
      sorter: (a, b) => a.priority - b.priority,
    },
    {
      title: "Max Purchase Price",
      dataIndex: "maxPurchasePrice",
      key: "maxPurchasePrice",
      render: (value: number) => <MoneyDisplay amount={value} />,
      sorter: (a, b) => a.maxPurchasePrice - b.maxPurchasePrice,
    },
    {
      title: "Credits Threshold",
      dataIndex: "creditsThreshold",
      key: "creditsThreshold",
      render: (value: number) => <MoneyDisplay amount={value} />,
      sorter: (a, b) => a.creditsThreshold - b.creditsThreshold,
    },
    {
      title: "Range Min",
      dataIndex: "rangeMin",
      key: "rangeMin",
      sorter: (a, b) => a.rangeMin - b.rangeMin,
    },
    {
      title: "Cargo Min",
      dataIndex: "cargoMin",
      key: "cargoMin",
      sorter: (a, b) => a.cargoMin - b.cargoMin,
    },
    {
      title: "Siphon",
      dataIndex: "siphon",
      key: "siphon",
      render: renderBool,
      filters: boolFilters,
      onFilter: (value, record) => record.siphon === value,
      sorter: (a, b) => Number(a.siphon) - Number(b.siphon),
    },
    {
      title: "Warp Drive",
      dataIndex: "warpDrive",
      key: "warpDrive",
      render: renderBool,
      filters: boolFilters,
      onFilter: (value, record) => record.warpDrive === value,
      sorter: (a, b) => Number(a.warpDrive) - Number(b.warpDrive),
    },
    {
      title: "Survey",
      dataIndex: "survey",
      key: "survey",
      render: renderBool,
      filters: boolFilters,
      onFilter: (value, record) => record.survey === value,
      sorter: (a, b) => Number(a.survey) - Number(b.survey),
    },
    {
      title: "Extractor",
      dataIndex: "extractor",
      key: "extractor",
      render: renderBool,
      filters: boolFilters,
      onFilter: (value, record) => record.extractor === value,
      sorter: (a, b) => Number(a.extractor) - Number(b.extractor),
    },
    {
      title: "Disabled",
      dataIndex: "disabled",
      key: "disabled",
      render: renderBool,
      filters: boolFilters,
      onFilter: (value, record) => record.disabled === value,
      sorter: (a, b) => Number(a.disabled) - Number(b.disabled),
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="Ships to Purchase" />
      <Space>
        <h1>Ships to Purchase {items.length}</h1>
        <Button
          onClick={() => {
            refetch();
          }}
        >
          Refresh
        </Button>
        <Spin spinning={loading || dataState != "complete"} />
      </Space>
      <Table<Assignment>
        dataSource={items}
        rowKey="id"
        columns={columns}
        scroll={{ x: "max-content" }}
        pagination={{ pageSize: 25, showSizeChanger: true }}
      />
    </div>
  );
}
