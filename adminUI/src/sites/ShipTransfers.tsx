import { useQuery } from "@apollo/client/react";
import { Button, Divider, Space, Switch, Table, TableProps } from "antd";
import { useState } from "react";
import { Link } from "react-router-dom";
import PageTitle from "../features/PageTitle";
import { GetShipTransfersQuery, ShipTransferState } from "../gql/graphql";
import { GET_SHIP_TRANSFERS } from "../graphql/queries";

type ShipTransferItem = GetShipTransfersQuery["shipTransfers"]["items"][number];

function ShipTransfers() {
  const [page, setPage] = useState<number | undefined>(1);
  const [pageSize, setPageSize] = useState<number | undefined>(100);
  const [clientPagination, setClientPagination] = useState(false);

  const { loading, error, data, previousData, refetch } = useQuery(
    GET_SHIP_TRANSFERS,
    {
      variables: { page, pageSize },
    },
  );

  if (error) return <p>Error: {error.message}</p>;

  const totalCount =
    data?.shipTransfers.totalCount ??
    previousData?.shipTransfers.totalCount ??
    0;
  const transfers =
    data?.shipTransfers.items ?? previousData?.shipTransfers.items ?? [];

  const columns: TableProps<ShipTransferItem>["columns"] = [
    {
      title: "ID",
      dataIndex: "id",
      key: "id",
      render: (id: number) => <Link to={`/shipTransfers/${id}`}>{id}</Link>,
      sorter: (a, b) => a.id - b.id,
      defaultSortOrder: "descend",
    },
    {
      title: "Ship",
      dataIndex: "shipSymbol",
      key: "shipSymbol",
      render: (symbol: string) => <Link to={`/ships/${symbol}`}>{symbol}</Link>,
      sorter: (a, b) => a.shipSymbol.localeCompare(b.shipSymbol),
    },
    {
      title: "State",
      dataIndex: "state",
      key: "state",
      sorter: (a, b) => a.state.localeCompare(b.state),
      filters: Object.values(ShipTransferState).map((state) => ({
        text: state,
        value: state,
      })),
      onFilter: (value, record) => record.state === value,
    },
    {
      title: "Fleet",
      dataIndex: "fleetId",
      key: "fleetId",
      render: (id: number) => <Link to={`/fleets/${id}`}>{id}</Link>,
      sorter: (a, b) => a.fleetId - b.fleetId,
    },
    {
      title: "Assignment",
      dataIndex: "assignmentId",
      key: "assignmentId",
      sorter: (a, b) => a.assignmentId - b.assignmentId,
    },
    {
      title: "Reservation",
      dataIndex: "reservationId",
      key: "reservationId",
      render: (id: number) => <Link to={`/reservedFunds`}>{id}</Link>,
      sorter: (a, b) => a.reservationId - b.reservationId,
    },
    {
      title: "Start System",
      dataIndex: "startSystemSymbol",
      key: "startSystemSymbol",
      render: (symbol: string) => <Link to={`/system/${symbol}`}>{symbol}</Link>,
      sorter: (a, b) => a.startSystemSymbol.localeCompare(b.startSystemSymbol),
    },
    {
      title: "End System",
      dataIndex: "endSystemSymbol",
      key: "endSystemSymbol",
      render: (symbol: string) => <Link to={`/system/${symbol}`}>{symbol}</Link>,
      sorter: (a, b) => a.endSystemSymbol.localeCompare(b.endSystemSymbol),
    },
    {
      title: "Created At",
      dataIndex: "createdAt",
      key: "createdAt",
      render: (date: string) => new Date(date).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime(),
    },
    {
      title: "Updated At",
      dataIndex: "updatedAt",
      key: "updatedAt",
      render: (date: string) => new Date(date).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.updatedAt).getTime() - new Date(b.updatedAt).getTime(),
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title={`Ship Transfers ${totalCount}`} />
      <Space>
        <h1 className="scroll-m-20 text-center text-3xl font-bold tracking-tight text-balance">
          Ship Transfers {totalCount}
        </h1>
        <Button onClick={() => refetch()}>Refresh</Button>
        Client Pagination{" "}
        <Switch
          checked={clientPagination}
          onChange={(checked) => {
            setClientPagination(checked);
            if (checked) {
              setPage(undefined);
              setPageSize(undefined);
            }
          }}
        />
      </Space>
      <Divider />
      <Table
        dataSource={transfers}
        columns={columns}
        rowKey="id"
        loading={loading}
        scroll={{ x: "max-content" }}
        pagination={{
          current: page,
          pageSize,
          total: clientPagination ? undefined : totalCount,
          showSizeChanger: true,
          pageSizeOptions: ["10", "20", "50", "100", "200", "500", "1000"],
          showTotal: (total, range) => `${range[0]}-${range[1]} of ${total}`,
        }}
        onChange={(pagination) => {
          if (clientPagination) return;
          setPage(pagination.current ?? 1);
          setPageSize(pagination.pageSize ?? 100);
        }}
      />
    </div>
  );
}

export default ShipTransfers;
