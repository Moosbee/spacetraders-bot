import { Button, Space, Table, TableProps } from "antd";
import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import WaypointLink from "../features/WaypointLink";
import { ConstructionShipment } from "../models/Construction";
import { backendUrl } from "../store";

function ConstructionShipments() {
  const [constructionShipments, setConstructionShipments] = useState<
    ConstructionShipment[]
  >([]);

  useEffect(() => {
    fetch(`http://${backendUrl}/construction/shipments`)
      .then((response) => response.json())
      .then((data) => {
        console.log("constructionShipments", data);

        setConstructionShipments(data);
      });
  }, []);

  const columns: TableProps<ConstructionShipment>["columns"] = [
    {
      title: "id",
      dataIndex: "id",
      key: "id",
      sorter: (a, b) => a.id - b.id,
      defaultSortOrder: "descend",
    },
    {
      title: "material_id",
      dataIndex: "material_id",
      key: "material_id",
      sorter: (a, b) => a.material_id - b.material_id,
    },
    {
      title: "construction_site_waypoint",
      dataIndex: "construction_site_waypoint",
      key: "construction_site_waypoint",
      sorter: (a, b) =>
        a.construction_site_waypoint.localeCompare(
          b.construction_site_waypoint
        ),
      render: (symbol: string) => (
        <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
      ),
    },
    {
      title: "ship_symbol",
      dataIndex: "ship_symbol",
      key: "ship_symbol",
      sorter: (a, b) => a.ship_symbol.localeCompare(b.ship_symbol),
      render: (symbol: string) => <Link to={`/ships/${symbol}`}>{symbol}</Link>,
      filters: [
        ...new Set(constructionShipments.map((t) => t.ship_symbol)),
      ].map((t) => ({
        text: t,
        value: t,
      })),
      onFilter: (value, record) => record.ship_symbol === value,
    },
    {
      title: "trade_symbol",
      dataIndex: "trade_symbol",
      key: "trade_symbol",
      sorter: (a, b) => a.trade_symbol.localeCompare(b.trade_symbol),
      filters: [
        ...new Set(constructionShipments.map((t) => t.trade_symbol)),
      ].map((t) => ({
        text: t,
        value: t,
      })),
      onFilter: (value, record) => record.trade_symbol === value,
    },
    {
      title: "units",
      dataIndex: "units",
      key: "units",
      sorter: (a, b) => a.units - b.units,
      align: "right",
    },
    {
      title: "purchase_waypoint",
      dataIndex: "purchase_waypoint",
      key: "purchase_waypoint",
      sorter: (a, b) => a.purchase_waypoint.localeCompare(b.purchase_waypoint),
      render: (symbol: string) => (
        <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
      ),
    },
    {
      title: "Status",
      dataIndex: "status",
      key: "status",
      render: (value) => value,
      sorter: (a, b) => (a.status === b.status ? 0 : a.status ? -1 : 1),
      filters: [
        { text: "Delivered", value: "Delivered" },
        { text: "InTransit", value: "InTransit" },
        { text: "Failed", value: "Failed" },
      ],
      onFilter: (value, record) => record.status === value,
    },

    {
      title: "created_at",
      dataIndex: "created_at",
      key: "created_at",
      render: (date: string) => new Date(date).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.created_at).getTime() - new Date(b.created_at).getTime(),
    },
    {
      title: "updated_at",
      dataIndex: "updated_at",
      key: "updated_at",
      render: (date: string) => new Date(date).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.updated_at).getTime() - new Date(b.updated_at).getTime(),
    },
    // {
    //   title: "sum",
    //   dataIndex: "sum",
    //   key: "sum",
    //   render: (value) => <MoneyDisplay amount={value || 0} />,
    //   sorter: (a, b) => (a.sum || 0) - (b.sum || 0),
    //   align: "right",
    // },
    {
      title: "expenses",
      dataIndex: "expenses",
      key: "expenses",
      render: (value) => <MoneyDisplay amount={value || 0} />,
      sorter: (a, b) => (a.expenses || 0) - (b.expenses || 0),
      align: "right",
    },
    {
      title: "income",
      dataIndex: "income",
      key: "income",
      render: (value) => <MoneyDisplay amount={value || 0} />,
      sorter: (a, b) => (a.income || 0) - (b.income || 0),
      align: "right",
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="Construction Shipments" />
      <Space>
        <h1>Construction Shipments</h1>
        <Button
          onClick={() => {
            fetch(`http://${backendUrl}/construction/shipments`)
              .then((response) => response.json())
              .then((data) => {
                console.log("constructionShipments", data);

                setConstructionShipments(data);
              });
          }}
        >
          Refresh
        </Button>
      </Space>
      <Table
        dataSource={constructionShipments || []}
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

export default ConstructionShipments;
