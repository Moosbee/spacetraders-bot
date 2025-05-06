import { Button, Progress, Space, Table, TableProps } from "antd";
import { useEffect, useState } from "react";
import { backendUrl } from "../data";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import { ConstructionMaterialSummary } from "../models/Construction";

function ConstructionMaterials() {
  const [constructionMaterials, setConstructionMaterials] = useState<
    ConstructionMaterialSummary[]
  >([]);

  useEffect(() => {
    fetch(`http://${backendUrl}/construction/materials`)
      .then((response) => response.json())
      .then((data) => {
        console.log("ConstructionMaterials", data);

        setConstructionMaterials(data);
      });
  }, []);

  const columns: TableProps<ConstructionMaterialSummary>["columns"] = [
    {
      title: "id",
      dataIndex: "id",
      key: "id",
      sorter: (a, b) => a.id - b.id,
      defaultSortOrder: "descend",
    },
    {
      title: "waypoint_symbol",
      dataIndex: "waypoint_symbol",
      key: "waypoint_symbol",
      sorter: (a, b) => a.waypoint_symbol.localeCompare(b.waypoint_symbol),
    },
    {
      title: "trade_symbol",
      dataIndex: "trade_symbol",
      key: "trade_symbol",
      sorter: (a, b) => a.trade_symbol.localeCompare(b.trade_symbol),
    },
    {
      title: "required",
      dataIndex: "required",
      key: "required",
      sorter: (a, b) => a.required - b.required,
      align: "right",
    },
    {
      title: "fulfilled",
      dataIndex: "fulfilled",
      key: "fulfilled",
      sorter: (a, b) => a.fulfilled - b.fulfilled,
      align: "right",
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
    {
      title: "Percent",
      dataIndex: "",
      key: "percent",
      render: (_, record) => (
        <>
          <Progress
            percent={(record.fulfilled / record.required) * 100}
            size={"small"}
          />
          {/* {" "} */}
          {/* {((record.fulfilled / record.required) * 100).toFixed(2)}% */}
        </>
      ),
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
      <PageTitle title="ConstructionMaterials" />
      <Space>
        <h1>ConstructionMaterials</h1>
        <Button
          onClick={() => {
            fetch(`http://${backendUrl}/construction/materials`)
              .then((response) => response.json())
              .then((data) => {
                console.log("ConstructionMaterials", data);

                setConstructionMaterials(data);
              });
          }}
        >
          Refresh
        </Button>
      </Space>
      <Table
        dataSource={constructionMaterials || []}
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

export default ConstructionMaterials;
