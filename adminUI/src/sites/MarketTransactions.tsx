import { Button, Space } from "antd";
import { useEffect, useState } from "react";
import { backendUrl } from "../data";
import PageTitle from "../features/PageTitle";
import TransactionTable from "../features/TransactionTableOld/TransactionTable";
import { Transaction } from "../models/Transaction";

function MarketTransactions() {
  const [transactions, setTransactions] = useState<Transaction[] | null>(null);

  useEffect(() => {
    fetch(`http://${backendUrl}/transactions`)
      .then((response) => response.json())
      .then((data) => {
        console.log("tradeRoutes", data);

        setTransactions(data);
      });
  }, []);

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title={`Market Transactions ${transactions?.length}`} />
      <Space>
        <h1>Market Transactions {transactions?.length}</h1>
        <Button
          onClick={() => {
            fetch(`http://${backendUrl}/transactions`)
              .then((response) => response.json())
              .then((data) => {
                console.log("Contract", data);

                setTransactions(data);
              });
          }}
        >
          Refresh
        </Button>
      </Space>
      <TransactionTable transactions={transactions || []} />
    </div>
  );
}

export default MarketTransactions;
