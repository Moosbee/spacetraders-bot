import { useQuery } from "@apollo/client/react";
import { Button, Divider, Space } from "antd";
import { useState } from "react";
import PageTitle from "../features/PageTitle";
import TransactionTable from "../features/TransactionTable/TransactionTable";
import { GET_MARKET_TRANSACTIONS } from "../graphql/queries";

function MarketTransactions() {
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(100);

  const { loading, error, data, previousData, refetch } = useQuery(
    GET_MARKET_TRANSACTIONS,
    {
      variables: { page, pageSize },
    },
  );

  if (error) return <p>Error: {error.message}</p>;

  const totalCount =
    data?.marketTransactions.totalCount ??
    previousData?.marketTransactions.totalCount ??
    0;
  const transactions =
    data?.marketTransactions.items ??
    previousData?.marketTransactions.items ??
    [];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title={`Market Transactions ${totalCount}`} />
      <Space>
        <h1 className="scroll-m-20 text-center text-3xl font-bold tracking-tight text-balance">
          Market Transactions {totalCount}
        </h1>
        <Button onClick={() => refetch()}>Refresh</Button>
      </Space>
      <Divider />
      <TransactionTable
        transactions={transactions}
        loading={loading}
        pagination={{
          current: page,
          pageSize,
          total: totalCount,
          showSizeChanger: true,
          pageSizeOptions: ["10", "20", "50", "100", "200", "500", "1000"],
          showTotal: (total, range) => `${range[0]}-${range[1]} of ${total}`,
        }}
        onChange={(pagination) => {
          setPage(pagination.current ?? 1);
          setPageSize(pagination.pageSize ?? 100);
        }}
      />
    </div>
  );
}

export default MarketTransactions;
