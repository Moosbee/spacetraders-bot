import { useQuery } from "@apollo/client/react";
import { Button, Space, Table, TableProps } from "antd";
import PageTitle from "../features/PageTitle";
import Timer from "../features/Timer/Timer";
import { GetAllSurveysQuery, SurveySize, TradeSymbol } from "../gql/graphql";
import { GET_ALL_SURVEYS } from "../graphql/queries";

type GQLSurvey = GetAllSurveysQuery["surveys"][number];

export default function Surveys() {
  const { loading, error, data, dataState, refetch } =
    useQuery(GET_ALL_SURVEYS);

  if (dataState != "complete") return <p>Loading...</p>;
  if (error) return <p>Error: {error.message}</p>;

  const columns: TableProps<GQLSurvey>["columns"] = [
    {
      title: "Signature",
      dataIndex: "signature",
      key: "signature",
      sorter: (a, b) => a.signature.localeCompare(b.signature),
      render: (signature: string, record) => (
        <span
          style={{
            color:
              record.exhaustedSince ||
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
            !record.exhaustedSince &&
            new Date(record.expiration).getTime() > Date.now()
          );
        } else {
          return (
            !!record.exhaustedSince ||
            new Date(record.expiration).getTime() < Date.now()
          );
        }
      },
    },
    {
      title: "Waypoint Symbol",
      dataIndex: "waypointSymbol",
      key: "waypointSymbol",
      sorter: (a, b) => a.waypointSymbol.localeCompare(b.waypointSymbol),
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
      filters: Object.values(SurveySize).map((size) => ({
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
      dataIndex: "exhaustedSince",
      key: "exhaustedSince",
      sorter: (a, b) =>
        (a.exhaustedSince ? new Date(a.exhaustedSince).getTime() : 0) -
        (b.exhaustedSince ? new Date(b.exhaustedSince).getTime() : 0),
      render: (exhaustedSince) =>
        exhaustedSince ? new Date(exhaustedSince).toLocaleString() : "N/A",
    },
    {
      title: "Created At",
      dataIndex: "createdAt",
      key: "createdAt",
      sorter: (a, b) => a.createdAt.localeCompare(b.createdAt),
      render: (createdAt) => new Date(createdAt).toLocaleString(),
      defaultSortOrder: "descend",
    },
    {
      title: "Updated At",
      dataIndex: "updatedAt",
      key: "updatedAt",
      sorter: (a, b) => a.updatedAt.localeCompare(b.updatedAt),
      render: (updatedAt) => new Date(updatedAt).toLocaleString(),
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="surveys" />
      <Space>
        <h1 className="scroll-m-20 text-center text-3xl font-bold tracking-tight text-balance">
          Surveys {data.surveys.length}
        </h1>
        <Button
          onClick={() => {
            refetch();
          }}
        >
          Refresh
        </Button>
      </Space>
      <Table
        loading={loading}
        dataSource={data.surveys}
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
