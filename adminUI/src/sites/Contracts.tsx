import { Button, Divider, Space, Table, TableProps } from "antd";
import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import WaypointLink from "../features/WaypointLink";
import { Contract } from "../models/Contract";
import { ContractShipment } from "../models/SQLContract";
import { backendUrl } from "../store";

function Contracts() {
  const [contractResp, setContract] = useState<Contract[] | null>(null);
  const [runningContractShipments, setRunningContractShipments] = useState<
    ContractShipment[] | null
  >(null);

  useEffect(() => {
    fetch(`http://${backendUrl}/contracts`)
      .then((response) => response.json())
      .then((data) => {
        console.log("setContract", data);

        setContract(data);
      });
    fetch(`http://${backendUrl}/insights/contract/shipments`)
      .then((response) => response.json())
      .then((data) => {
        console.log("setRunningContractShipments", data);

        setRunningContractShipments(data.shipments);
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
            fetch(`http://${backendUrl}/insights/contract/shipments`)
              .then((response) => response.json())
              .then((data) => {
                console.log("setRunningContractShipments", data);

                setRunningContractShipments(data.shipments);
              });
          }}
        >
          Refresh
        </Button>
      </Space>
      <Table
        title={() => "Running Contract Shipments"}
        size="small"
        rowKey={(id) => id.id}
        columns={[
          {
            title: "ID",
            dataIndex: "id",
            key: "id",
          },
          {
            title: "Ship Symbol",
            dataIndex: "ship_symbol",
            key: "ship_symbol",
          },
          {
            title: "Trade Symbol",
            dataIndex: "trade_symbol",
            key: "trade_symbol",
          },
          {
            title: "Units",
            dataIndex: "units",
            key: "units",
          },
          {
            title: "Destination Symbol",
            dataIndex: "destination_symbol",
            key: "destination_symbol",
            render: (symbol) => (
              <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
            ),
          },
          {
            title: "Purchase Symbol",
            dataIndex: "purchase_symbol",
            key: "purchase_symbol",
            render: (symbol) => (
              <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
            ),
          },
          {
            title: "Created At",
            dataIndex: "created_at",
            key: "created_at",
            render: (date) => new Date(date).toLocaleString(),
          },
          {
            title: "Updated At",
            dataIndex: "updated_at",
            key: "updated_at",
            render: (date) => new Date(date).toLocaleString(),
          },
          {
            title: "Status",
            dataIndex: "status",
            key: "status",
          },
        ]}
        dataSource={runningContractShipments || []}
      ></Table>
      <Divider />
      <Table
        title={() => "Contracts"}
        dataSource={contractResp || []}
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

export default Contracts;
