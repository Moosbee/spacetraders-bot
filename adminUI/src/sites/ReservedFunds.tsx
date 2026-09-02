import { useQuery } from "@apollo/client/react";
import {
  Button,
  Descriptions,
  Divider,
  Flex,
  Space,
  Table,
  TableProps,
} from "antd";
import { Link } from "react-router-dom";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import { GetReservedFundsQuery } from "../gql/graphql";
import { GET_RESERVED_FUNDS } from "../graphql/queries";

type GQLReservedFund = GetReservedFundsQuery["budget"]["reservations"][number];
type GQLReservedFundSimple =
  GetReservedFundsQuery["reservedFunds"]["items"][number];

export default function ReservedFunds() {
  const { loading, error, data, refetch } = useQuery(GET_RESERVED_FUNDS);

  if (error) return <p>Error: {error.message}</p>;

  const budget = data?.budget;
  const currentReservations = budget?.reservations ?? [];
  const allReservations = data?.reservedFunds.items ?? [];

  const fullTableColumns: TableProps<GQLReservedFund>["columns"] = [
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
      dataIndex: "actualAmount",
      key: "actualAmount",
      render: (value) => <MoneyDisplay amount={value} />,
    },
    {
      title: "Discrepancy",
      key: "discrepancy",
      render: (_, record) => (
        <MoneyDisplay amount={record.amount - record.actualAmount} />
      ),
    },
    {
      title: "Status",
      dataIndex: "status",
      key: "status",
    },
    {
      title: "Contract",
      key: "contract",
      render: (_, record) => {
        const contracts = record.contract?.items ?? [];
        if (contracts.length === 0) return "N/A";
        return (
          <Space size={4} wrap>
            {contracts.map((contract) => (
              <Link key={contract.id} to={`/contracts/${contract.id}`}>
                {contract.id}
              </Link>
            ))}
          </Space>
        );
      },
    },
    {
      title: "Trade Route",
      key: "tradeRoute",
      render: (_, record) => {
        const routes = record.tradeRoute?.items ?? [];
        if (routes.length === 0) return "N/A";
        return routes.map((route) => route.id).join(", ");
      },
    },
    {
      title: "Constr. Shipment",
      key: "constructionShipment",
      render: (_, record) => {
        const shipments = record.constructionShipment?.items ?? [];
        if (shipments.length === 0) return "N/A";
        return shipments.map((shipment) => shipment.id).join(", ");
      },
    },
    {
      title: "Created At",
      dataIndex: "createdAt",
      key: "createdAt",
      render: (date) => new Date(date).toLocaleString(),
    },
    {
      title: "Updated At",
      dataIndex: "updatedAt",
      key: "updatedAt",
      render: (date) => new Date(date).toLocaleString(),
    },
  ];

  const tableColumns: TableProps<GQLReservedFundSimple>["columns"] = [
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
      dataIndex: "actualAmount",
      key: "actualAmount",
      render: (value) => <MoneyDisplay amount={value} />,
    },
    {
      title: "Discrepancy",
      key: "discrepancy",
      render: (_, record) => (
        <MoneyDisplay amount={record.amount - record.actualAmount} />
      ),
    },
    {
      title: "Status",
      dataIndex: "status",
      key: "status",
    },
    {
      title: "Created At",
      dataIndex: "createdAt",
      key: "createdAt",
      render: (date) => new Date(date).toLocaleString(),
    },
    {
      title: "Updated At",
      dataIndex: "updatedAt",
      key: "updatedAt",
      render: (date) => new Date(date).toLocaleString(),
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="Reserved Funds" />
      <Space>
        <h1 className="scroll-m-20 text-center text-3xl font-bold tracking-tight text-balance">
          Reserved Funds
        </h1>
        <Button onClick={() => refetch()}>Refresh</Button>
      </Space>
      <Divider />
      <Flex gap={12} vertical>
        <Descriptions
          bordered
          items={[
            {
              label: "Current Funds",
              children: <MoneyDisplay amount={budget?.currentFunds ?? 0} />,
            },
            {
              label: "Iron Reserve",
              children: <MoneyDisplay amount={budget?.ironReserve ?? 0} />,
            },
            {
              label: "Reserved Amount",
              children: <MoneyDisplay amount={budget?.reservedAmount ?? 0} />,
            },
            {
              label: "Spendable",
              children: <MoneyDisplay amount={budget?.spendable ?? 0} />,
            },
            {
              label: "Current Reservations",
              children: <span>{currentReservations.length}</span>,
            },
          ]}
        />
        <Table
          title={() => "Current Reservations"}
          dataSource={currentReservations}
          columns={fullTableColumns}
          rowKey={(record) => record.id}
          loading={loading}
        />
        <Table
          title={() => "All Reservations"}
          dataSource={allReservations}
          columns={tableColumns}
          rowKey={(record) => record.id}
          loading={loading}
        />
      </Flex>
    </div>
  );
}
