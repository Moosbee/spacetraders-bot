import { Button, Space, Table, TableProps } from "antd";
import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { backendUrl } from "../data";
import PageTitle from "../features/PageTitle";
import { SystemType } from "../models/api";
import { SQLSystem } from "../models/SQLSystem";

function Systems() {
  const [systems, setSystems] = useState<SQLSystem[] | null>(null);

  useEffect(() => {
    fetch(`http://${backendUrl}/systems`)
      .then((response) => response.json())
      .then((data) => {
        console.log("systems", data);

        setSystems(data);
      });
  }, []);

  // symbol: string;
  // sector_symbol: string;
  // system_type: SystemType;
  // x: number;
  // y: number;

  const columns: TableProps<SQLSystem>["columns"] = [
    {
      title: "Symbol",
      dataIndex: "symbol",
      key: "symbol",
      render: (symbol) => <Link to={`/system/${symbol}`}>{symbol}</Link>,
      sorter: (a, b) => a.symbol.localeCompare(b.symbol),
    },
    {
      title: "Sector Symbol",
      dataIndex: "sector_symbol",
      key: "sector_symbol",
      sorter: (a, b) => a.sector_symbol.localeCompare(b.sector_symbol),
      filters: [...new Set(systems?.map((s) => s.sector_symbol) || [])].map(
        (s) => ({
          text: s,
          value: s,
        })
      ),
      onFilter: (value, record) => record.sector_symbol === value,
    },
    {
      title: "System Type",
      dataIndex: "system_type",
      key: "system_type",
      sorter: (a, b) => a.system_type.localeCompare(b.system_type),
      filters: Object.values(SystemType).map((s) => ({
        text: s,
        value: s,
      })),
      onFilter: (value, record) => record.system_type === value,
    },
    {
      title: "X",
      dataIndex: "x",
      key: "x",
      sorter: (a, b) => a.x - b.x,
    },
    {
      title: "Y",
      dataIndex: "y",
      key: "y",
      sorter: (a, b) => a.y - b.y,
    },
    {
      title: "Waypoints",
      dataIndex: "waypoints",
      key: "waypoints",
      sorter: (a, b) => (a.waypoints || 0) - (b.waypoints || 0),
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="Systems" />
      <Space>
        <h1>Systems {systems?.length}</h1>
        <Button
          onClick={() => {
            fetch(`http://${backendUrl}/systems`)
              .then((response) => response.json())
              .then((data) => {
                console.log("systems", data);

                setSystems(data);
              });
          }}
        >
          Refresh
        </Button>
      </Space>
      <Table
        dataSource={systems || []}
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

export default Systems;
