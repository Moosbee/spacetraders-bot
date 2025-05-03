import { Button, Space, Table, TableProps } from "antd";
import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import WaypointLink from "../features/WaypointLink";
import { DbAgent } from "../models/Agent";
import { FactionSymbol } from "../models/api";
import { backendUrl } from "../MyApp";

function Agents() {
  const [agents, setAgents] = useState<DbAgent[] | null>(null);

  useEffect(() => {
    fetch(`http://${backendUrl}/agents`)
      .then((response) => response.json())
      .then((data) => {
        console.log("agents", data);

        setAgents(data);
      });
  }, []);

  const columns: TableProps<DbAgent>["columns"] = [
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
      title: "Account ID",
      dataIndex: "account_id",
      key: "account_id",
      render: (id?: string) => id || "N/A",
      sorter: (a, b) => (a.account_id ?? "").localeCompare(b.account_id ?? ""),
      filters: [
        {
          text: "Yes",
          value: "Yes",
        },
        {
          text: "No",
          value: "No",
        },
      ],
      onFilter: (value: boolean | React.Key, record) =>
        (value === "No" && !record.account_id) ||
        (value === "Yes" && !!record.account_id),
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
      dataIndex: "starting_faction",
      key: "starting_faction",
      sorter: (a, b) => a.starting_faction.localeCompare(b.starting_faction),
      filters: Object.values(FactionSymbol).map((f) => ({
        text: f,
        value: f,
      })),
      onFilter: (value, record) => record.starting_faction === value,
    },
    {
      title: "Ship Count",
      dataIndex: "ship_count",
      key: "ship_count",
      sorter: (a, b) => a.ship_count - b.ship_count,
    },
    {
      title: "Last Updated",
      dataIndex: "created_at",
      key: "created_at",
      render: (date: string) => new Date(date).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.created_at).getTime() - new Date(b.created_at).getTime(),
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="Agents" />
      <Space>
        <h1>Agents {agents?.length}</h1>
        <Button
          onClick={() => {
            fetch(`http://${backendUrl}/agents`)
              .then((response) => response.json())
              .then((data) => {
                console.log("agents", data);

                setAgents(data);
              });
          }}
        >
          Refresh
        </Button>
      </Space>
      <Table
        dataSource={agents || []}
        columns={columns}
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
