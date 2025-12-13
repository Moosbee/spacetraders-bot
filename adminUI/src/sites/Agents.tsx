import { useQuery } from "@apollo/client/react";
import { Button, Space, Table, TableProps } from "antd";
import { Link } from "react-router-dom";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import WaypointLink from "../features/WaypointLink";
import { GetAllAgentsQuery } from "../gql/graphql";
import { GET_ALL_AGENTS } from "../graphql/queries";
import { FactionSymbol } from "../models/api";

type GQLAgent = GetAllAgentsQuery["agents"][number];

function Agents() {
  const { loading, error, data, dataState, refetch } = useQuery(GET_ALL_AGENTS);

  if (dataState != "complete") return <p>Loading...</p>;
  if (error) return <p>Error: {error.message}</p>;

  const columns: TableProps<GQLAgent>["columns"] = [
    {
      title: "Agent Symbol",
      dataIndex: "symbol",
      key: "symbol",
      render: (symbol: string) => (
        <Link to={`/agents/${symbol}`}>{symbol}</Link>
      ),
      sorter: (a, b) => a.symbol.localeCompare(b.symbol),
    },
    {
      title: "Headquarters",
      dataIndex: "headquarters",
      key: "headquarters",
      render: (symbol: string) => (
        <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
      ),
      sorter: (a, b) => a.headquarters.localeCompare(b.headquarters),
    },
    {
      title: "Credits",
      dataIndex: "credits",
      key: "credits",
      render: (value) => <MoneyDisplay amount={value} />,
      align: "right",
      sorter: (a, b) => a.credits - b.credits,
    },
    {
      title: "Starting Faction",
      dataIndex: "startingFaction",
      key: "startingFaction",
      sorter: (a, b) => a.startingFaction.localeCompare(b.startingFaction),
      filters: Object.values(FactionSymbol).map((f) => ({
        text: f,
        value: f,
      })),
      onFilter: (value, record) => record.startingFaction === value,
    },
    {
      title: "Ship Count",
      dataIndex: "shipCount",
      key: "shipCount",
      sorter: (a, b) => a.shipCount - b.shipCount,
    },
    {
      title: "Last Updated",
      dataIndex: "createdAt",
      key: "createdAt",
      render: (date: string) => new Date(date).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime(),
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="Agents" />
      <Space>
        <h1 className="scroll-m-20 text-center text-3xl font-bold tracking-tight text-balance">
          Agents {data.agents?.length}
        </h1>
        <Button
          onClick={() => {
            refetch();
          }}
        >
          Refresh
        </Button>
      </Space>
      <Table
        dataSource={data.agents || []}
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

export default Agents;
