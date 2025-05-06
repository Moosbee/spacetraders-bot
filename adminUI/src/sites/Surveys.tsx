import { Button, Space, Table, TableProps } from "antd";
import { useEffect, useState } from "react";
import { backendUrl } from "../data";
import PageTitle from "../features/PageTitle";
import Timer from "../features/Timer/Timer";
import { SurveySizeEnum, TradeSymbol } from "../models/api";
import { Survey } from "../models/Survey";

export default function Surveys() {
  const [surveys, setSurveys] = useState<Survey[] | null>(null);

  useEffect(() => {
    fetch(`http://${backendUrl}/surveys`)
      .then((response) => response.json())
      .then((data) => {
        console.log("surveys", data);

        setSurveys(data);
      });
  }, []);

  // export interface Survey {
  //   ship_info_before: number;
  //   ship_info_after: number;
  //   signature: string;
  //   waypoint_symbol: string;
  //   deposits: TradeSymbol[];
  //   expiration: string;
  //   size: SurveySizeEnum;
  //   exhausted_since?: string;
  //   created_at: string;
  //   updated_at: string;
  // }

  const columns: TableProps<Survey>["columns"] = [
    {
      title: "Signature",
      dataIndex: "signature",
      key: "signature",
      sorter: (a, b) => a.signature.localeCompare(b.signature),
      render: (signature: string, record) => (
        <span
          style={{
            color:
              record.exhausted_since ||
              new Date(record.expiration).getTime() > Date.now()
                ? "currentColor"
                : "red",
          }}
        >
          {signature}
        </span>
      ),
      filters: [
        {
          text: "Valid",
          value: "valid",
        },
        {
          text: "Not Valid",
          value: "not_valid",
        },
      ],
      defaultFilteredValue: ["valid"],
      onFilter: (value, record) => {
        if (value === "valid") {
          return (
            !record.exhausted_since &&
            new Date(record.expiration).getTime() > Date.now()
          );
        } else {
          return (
            !!record.exhausted_since ||
            new Date(record.expiration).getTime() < Date.now()
          );
        }
      },
    },
    {
      title: "Waypoint Symbol",
      dataIndex: "waypoint_symbol",
      key: "waypoint_symbol",
      sorter: (a, b) => a.waypoint_symbol.localeCompare(b.waypoint_symbol),
    },
    {
      title: "Deposits",
      dataIndex: "deposits",
      key: "deposits",
      render: (deposits: TradeSymbol[]) => {
        const map: Partial<Record<TradeSymbol, number>> = {};
        deposits.forEach((deposit) => {
          map[deposit] = map[deposit] ? map[deposit] + 1 : 1;
        });

        return (
          <>
            {Object.entries(map)
              .sort((a, b) => b[1] - a[1])
              .map(([trade_symbol, count]) => ({
                trade_symbol,
                percentage: ((count / deposits.length) * 100).toFixed(2),
                count,
              }))
              .map(({ trade_symbol, percentage, count }) => (
                <div key={trade_symbol}>
                  {percentage}% {trade_symbol} ({count})
                </div>
              ))}
          </>
        );
      },
      filters: Object.values(TradeSymbol).map((trade_symbol) => ({
        text: trade_symbol,
        value: trade_symbol,
      })),
      onFilter: (value, record) =>
        record.deposits.includes(value as TradeSymbol),
    },

    {
      title: "Size",
      dataIndex: "size",
      key: "size",
      sorter: (a, b) => a.size.localeCompare(b.size),
      filters: Object.values(SurveySizeEnum).map((size) => ({
        text: size,
        value: size,
      })),
      onFilter: (value, record) => record.size === value,
    },
    {
      title: "Expiration",
      dataIndex: "expiration",
      key: "expiration",
      sorter: (a, b) => a.expiration.localeCompare(b.expiration),
      render: (expiration) => (
        <>
          {new Date(expiration).toLocaleString()}
          <br />
          <Timer time={expiration} />
        </>
      ),
    },
    {
      title: "Exhausted Since",
      dataIndex: "exhausted_since",
      key: "exhausted_since",
      sorter: (a, b) =>
        (a.exhausted_since ? new Date(a.exhausted_since).getTime() : 0) -
        (b.exhausted_since ? new Date(b.exhausted_since).getTime() : 0),
      render: (exhausted_since) =>
        exhausted_since ? new Date(exhausted_since).toLocaleString() : "N/A",
    },
    {
      title: "Created At",
      dataIndex: "created_at",
      key: "created_at",
      sorter: (a, b) => a.created_at.localeCompare(b.created_at),
      render: (created_at) => new Date(created_at).toLocaleString(),
      defaultSortOrder: "descend",
    },
    {
      title: "Updated At",
      dataIndex: "updated_at",
      key: "updated_at",
      sorter: (a, b) => a.updated_at.localeCompare(b.updated_at),
      render: (updated_at) => new Date(updated_at).toLocaleString(),
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="surveys" />
      <Space>
        <h1>surveys</h1>
        <Button
          onClick={() => {
            fetch(`http://${backendUrl}/surveys`)
              .then((response) => response.json())
              .then((data) => {
                console.log("surveys", data);

                setSurveys(data);
              });
          }}
        >
          Refresh
        </Button>
      </Space>
      <Table
        dataSource={surveys || []}
        columns={columns}
        rowKey="signature"
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
