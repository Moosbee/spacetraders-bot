import { Button, Descriptions, Flex, Space, Table, TableProps } from "antd";
import { useEffect, useState } from "react";
import { backendUrl } from "../data";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import { BudgetResponse, ReservedFund } from "../models/ReservedFund";

export default function ReservedFunds() {
  const [reservedFunds, setReservedFunds] = useState<BudgetResponse>({
    all_reservations: [],
    budget_info: {
      current_funds: 0,
      iron_reserve: 0,
      reserved_amount: 0,
      spendable: 0,
      reservations: [],
    },
  });

  useEffect(() => {
    fetch(`http://${backendUrl}/insights/budget`)
      .then((response) => response.json())
      .then((data) => {
        console.log("Budget", data);

        setReservedFunds(data);
      });
  }, []);

  const tableColumns: TableProps<ReservedFund>["columns"] = [
    {
      title: "ID",
      dataIndex: "id",
      key: "id",
      sorter: (a, b) => a.id - b.id,
      defaultSortOrder: "descend",
    },
    {
      title: "Amount",
      dataIndex: "amount",
      key: "amount",
      render: (value) => <MoneyDisplay amount={value} />,
    },
    {
      title: "Actual Amount",
      dataIndex: "actual_amount",
      key: "actual_amount",
      render: (value) => <MoneyDisplay amount={value} />,
    },
    {
      title: "Discrepancy",
      key: "discrepancy",
      render: (_, record) => (
        <MoneyDisplay amount={record.amount - record.actual_amount} />
      ),
    },
    {
      title: "Status",
      dataIndex: "status",
      key: "status",
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
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="Reserved Funds" />
      <Space>
        <h1>Reserved Funds</h1>{" "}
        <Button
          onClick={() => {
            fetch(`http://${backendUrl}/insights/budget`)
              .then((response) => response.json())
              .then((data) => {
                console.log("Budget", data);

                setReservedFunds(data);
              });
          }}
        >
          Refresh
        </Button>
      </Space>
      <Flex gap={12} vertical>
        <Descriptions
          bordered
          items={[
            {
              label: "Current Funds",
              children: (
                <span>
                  <MoneyDisplay
                    amount={reservedFunds.budget_info.current_funds}
                  />
                </span>
              ),
            },
            {
              label: "Iron Reserve",
              children: (
                <span>
                  <MoneyDisplay
                    amount={reservedFunds.budget_info.iron_reserve}
                  />
                </span>
              ),
            },
            {
              label: "Reserved Amount",
              children: (
                <span>
                  <MoneyDisplay
                    amount={reservedFunds.budget_info.reserved_amount}
                  />
                </span>
              ),
            },
            {
              label: "Spendable",
              children: (
                <span>
                  <MoneyDisplay amount={reservedFunds.budget_info.spendable} />
                </span>
              ),
            },
            {
              label: "Current Reservations",
              children: (
                <span>{reservedFunds.budget_info.reservations.length}</span>
              ),
            },
          ]}
        />
        <Table
          title={() => "Current Reservations"}
          dataSource={reservedFunds.budget_info.reservations}
          columns={tableColumns}
          rowKey={(record) => record.id}
        />
        <Table
          title={() => "All Reservations"}
          dataSource={reservedFunds.all_reservations}
          columns={tableColumns}
          rowKey={(record) => record.id}
        />
      </Flex>
    </div>
  );
}
