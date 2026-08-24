import { useQuery } from "@apollo/client/react";
import { Button, Divider, Space, Table, TableProps } from "antd";
import { Link } from "react-router-dom";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import WaypointLink from "../features/WaypointLink";
import { GetContractsQuery } from "../gql/graphql";
import { GET_CONTRACTS } from "../graphql/queries";

type GQLContract = GetContractsQuery["contracts"]["items"][number];

function Contracts() {
  const { loading, error, data, dataState, refetch } = useQuery(GET_CONTRACTS);

  if (dataState !== "complete") return <p>Loading...</p>;
  if (error) return <p>Error: {error.message}</p>;

  const contracts = data.contracts.items;
  const runningContractShipments = data.contractManager.runningShipments;

  const totalIncome = (c: GQLContract) =>
    c.onAccepted + c.onFulfilled + (c.marketTransactionSummary.allIncome ?? 0);
  const totalExpenses = (c: GQLContract) =>
    c.marketTransactionSummary.allExpenses ?? 0;
  const netProfit = (c: GQLContract) => totalIncome(c) - totalExpenses(c);

  const columns: TableProps<GQLContract>["columns"] = [
    {
      title: "ID",
      dataIndex: "id",
      key: "id",
      render: (value: string) => (
        <Link to={`/contracts/${value}`}>{value}</Link>
      ),
      sorter: (a, b) => a.id.localeCompare(b.id),
    },
    {
      title: "Faction",
      dataIndex: "factionSymbol",
      key: "factionSymbol",
      sorter: (a, b) => a.factionSymbol.localeCompare(b.factionSymbol),
    },
    {
      title: "Type",
      dataIndex: "contractType",
      key: "contractType",
      sorter: (a, b) => a.contractType.localeCompare(b.contractType),
    },
    {
      title: "Accepted",
      dataIndex: "accepted",
      key: "accepted",
      render: (value: boolean) => (value ? "Yes" : "No"),
      sorter: (a, b) => (a.accepted === b.accepted ? 0 : a.accepted ? -1 : 1),
      filters: [
        { text: "Yes", value: true },
        { text: "No", value: false },
      ],
      onFilter: (value, record) => record.accepted === value,
    },
    {
      title: "Fulfilled",
      dataIndex: "fulfilled",
      key: "fulfilled",
      render: (value: boolean) => (value ? "Yes" : "No"),
      sorter: (a, b) =>
        a.fulfilled === b.fulfilled ? 0 : a.fulfilled ? -1 : 1,
      filters: [
        { text: "Yes", value: true },
        { text: "No", value: false },
      ],
      onFilter: (value, record) => record.fulfilled === value,
    },
    {
      title: "Deadline to Accept",
      dataIndex: "deadlineToAccept",
      key: "deadlineToAccept",
      render: (value: string | null | undefined) =>
        value ? new Date(value).toLocaleString() : "N/A",
      sorter: (a, b) => {
        const av = a.deadlineToAccept
          ? new Date(a.deadlineToAccept).getTime()
          : 0;
        const bv = b.deadlineToAccept
          ? new Date(b.deadlineToAccept).getTime()
          : 0;
        return av - bv;
      },
      defaultSortOrder: "descend",
    },
    {
      title: "Deadline",
      dataIndex: "deadline",
      key: "deadline",
      render: (value: string) => new Date(value).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.deadline).getTime() - new Date(b.deadline).getTime(),
    },
    {
      title: "On Accepted",
      dataIndex: "onAccepted",
      key: "onAccepted",
      render: (value: number) => <MoneyDisplay amount={value} />,
      align: "right",
      sorter: (a, b) => a.onAccepted - b.onAccepted,
    },
    {
      title: "On Fulfilled",
      dataIndex: "onFulfilled",
      key: "onFulfilled",
      render: (value: number) => <MoneyDisplay amount={value} />,
      align: "right",
      sorter: (a, b) => a.onFulfilled - b.onFulfilled,
    },

    {
      title: "Total Profit",
      key: "totalProfit",
      render: (_, record) => <MoneyDisplay amount={totalIncome(record)} />,
      align: "right",
      sorter: (a, b) => totalIncome(a) - totalIncome(b),
    },
    {
      title: "Total Expenses",
      key: "totalExpenses",
      render: (_, record) => <MoneyDisplay amount={totalExpenses(record)} />,
      align: "right",
      sorter: (a, b) => totalExpenses(a) - totalExpenses(b),
    },
    {
      title: "Net Profit",
      key: "netProfit",
      render: (_, record) => (
        <MoneyDisplay
          amount={netProfit(record)}
          style={{ color: netProfit(record) < 0 ? "red" : "currentColor" }}
        />
      ),
      align: "right",
      sorter: (a, b) => netProfit(a) - netProfit(b),
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="Contracts" />
      <Space>
        <h1>Contracts</h1>
        <Button
          onClick={() => {
            refetch();
          }}
        >
          Refresh
        </Button>
      </Space>
      <Table
        title={() => "Running Contract Shipments"}
        size="small"
        loading={loading}
        rowKey={(id) => id.id}
        columns={[
          {
            title: "ID",
            dataIndex: "id",
            key: "id",
          },
          {
            title: "Ship Symbol",
            dataIndex: "shipSymbol",
            key: "shipSymbol",
          },
          {
            title: "Trade Symbol",
            dataIndex: "tradeSymbol",
            key: "tradeSymbol",
          },
          {
            title: "Units",
            dataIndex: "units",
            key: "units",
          },
          {
            title: "Destination Symbol",
            dataIndex: "destinationSymbol",
            key: "destinationSymbol",
            render: (symbol: string) => (
              <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
            ),
          },
          {
            title: "Purchase Symbol",
            dataIndex: "purchaseSymbol",
            key: "purchaseSymbol",
            render: (symbol: string) => (
              <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
            ),
          },
          {
            title: "Created At",
            dataIndex: "createdAt",
            key: "createdAt",
            render: (date: string) => new Date(date).toLocaleString(),
          },
          {
            title: "Updated At",
            dataIndex: "updatedAt",
            key: "updatedAt",
            render: (date: string) => new Date(date).toLocaleString(),
          },
          {
            title: "Status",
            dataIndex: "status",
            key: "status",
          },
        ]}
        dataSource={runningContractShipments}
      ></Table>
      <Divider />
      <Table
        title={() => "Contracts"}
        dataSource={contracts}
        columns={columns}
        loading={loading}
        rowKey="id"
        pagination={{
          showSizeChanger: true,
          pageSizeOptions: ["10", "20", "50", "100", "200", "500", "1000"],
          defaultPageSize: 100,
          showTotal: (total, range) => `${range[0]}-${range[1]} of ${total}`,
        }}
      />
    </div>
  );
}

export default Contracts;
