import { Button, Space, Table, TableProps } from "antd";
import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import { Contract } from "../models/Contract";
import { backendUrl } from "../store";

function Contracts() {
  const [contractResp, setContract] = useState<Contract[] | null>(null);

  useEffect(() => {
    fetch(`http://${backendUrl}/contracts`)
      .then((response) => response.json())
      .then((data) => {
        console.log("Contract", data);

        setContract(data);
      });
  }, []);

  const columns: TableProps<Contract>["columns"] = [
    {
      title: "ID",
      dataIndex: "id",
      key: "id",
      render: (value) => <Link to={`/contracts/${value}`}>{value}</Link>,
      sorter: (a, b) => a.id.localeCompare(b.id),
    },
    {
      title: "Faction",
      dataIndex: "faction_symbol",
      key: "faction_symbol",
      sorter: (a, b) => a.faction_symbol.localeCompare(b.faction_symbol),
    },
    {
      title: "Type",
      dataIndex: "contract_type",
      key: "contract_type",
      sorter: (a, b) => a.faction_symbol.localeCompare(b.faction_symbol),
    },
    {
      title: "Accepted",
      dataIndex: "accepted",
      key: "accepted",
      render: (value) => (value ? "Yes" : "No"), // Render boolean as "Yes" or "No"
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
      render: (value) => (value ? "Yes" : "No"), // Render boolean as "Yes" or "No"
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
      dataIndex: "deadline_to_accept",
      key: "deadline_to_accept",
      render: (value) => new Date(value).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.deadline_to_accept).getTime() -
        new Date(b.deadline_to_accept).getTime(),
      defaultSortOrder: "descend",
    },
    {
      title: "Deadline",
      dataIndex: "deadline",
      key: "deadline",
      render: (value) => new Date(value).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.deadline).getTime() - new Date(b.deadline).getTime(),
    },
    {
      title: "On Accepted",
      dataIndex: "on_accepted",
      key: "on_accepted",
      render: (value) => <MoneyDisplay amount={value} />,
      align: "right",
      sorter: (a, b) => a.on_accepted - b.on_accepted,
    },
    {
      title: "On Fulfilled",
      dataIndex: "on_fulfilled",
      key: "on_fulfilled",
      render: (value) => <MoneyDisplay amount={value} />,
      align: "right",
      sorter: (a, b) => a.on_fulfilled - b.on_fulfilled,
    },

    {
      title: "Total Profit",
      dataIndex: "totalprofit",
      key: "totalprofit",
      render: (value) => <MoneyDisplay amount={value} />,
      align: "right",
      sorter: (a, b) => a.totalprofit - b.totalprofit,
    },
    {
      title: "Total Expenses",
      dataIndex: "total_expenses",
      key: "total_expenses",
      render: (value) => <MoneyDisplay amount={value} />,
      align: "right",
      sorter: (a, b) => a.total_expenses - b.total_expenses,
    },
    {
      title: "Net Profit",
      dataIndex: "net_profit",
      key: "net_profit",
      render: (value) => (
        <MoneyDisplay
          amount={value}
          style={{ color: value < 0 ? "red" : "currentColor" }}
        />
      ),
      align: "right",
      sorter: (a, b) => a.net_profit - b.net_profit,
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="Contracts" />
      <Space>
        <h1>Contracts</h1>
        <Button
          onClick={() => {
            fetch(`http://${backendUrl}/contracts`)
              .then((response) => response.json())
              .then((data) => {
                console.log("Contract", data);

                setContract(data);
              });
          }}
        >
          Refresh
        </Button>
      </Space>
      <Table dataSource={contractResp || []} columns={columns} rowKey="id" />
    </div>
  );
}

export default Contracts;
