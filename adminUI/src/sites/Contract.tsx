import { useQuery } from "@apollo/client/react";
import { Button, Descriptions, Flex, Space, Table } from "antd";
import { Link, useParams } from "react-router-dom";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import TransactionTable from "../features/TransactionTable/TransactionTable";
import WaypointLink from "../features/WaypointLink";
import { GET_CONTRACT } from "../graphql/queries";

function Contract() {
  const { contractID } = useParams();

  const { loading, error, data, dataState, refetch } = useQuery(GET_CONTRACT, {
    variables: { contractId: contractID || "" },
  });

  if (dataState !== "complete") return <p>Loading...</p>;
  if (error) return <p>Error: {error.message}</p>;

  const contract = data.contract;

  const tradeUnitExpense = contract.marketTransactionSummary.expenses ?? 0;
  const travelExpense = contract.marketTransactionSummary.fuelExpenses ?? 0;
  const totalExpense = contract.marketTransactionSummary.allExpenses ?? 0;
  const totalReward = contract.marketTransactionSummary.allIncome ?? 0;
  const totalProfit =
    totalReward + contract.onAccepted + contract.onFulfilled - totalExpense;

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title={`Contract ${contractID}`} />
      <Space>
        <h1>Contract {contractID}</h1>
        <Button
          onClick={() => {
            refetch();
          }}
        >
          Reload
        </Button>
      </Space>
      <Flex gap={12} vertical>
        <Flex gap={8} justify="space-between" align="center">
          <Descriptions
            bordered
            // size="small"
            column={3}
            // layout="vertical"
            items={[
              {
                label: "Type",
                key: "type",
                children: contract.contractType,
              },
              {
                label: "Accepted",
                key: "accepted",
                children: contract.accepted ? "Yes" : "No",
              },
              {
                label: "deadlineToAccept",
                key: "deadlineToAccept",
                children: contract.deadlineToAccept
                  ? new Date(contract.deadlineToAccept).toLocaleString()
                  : "N/A",
              },

              {
                label: "Faction Symbol",
                key: "factionSymbol",
                children: contract.factionSymbol,
              },
              {
                label: "Fulfilled",
                key: "fulfilled",
                children: contract.fulfilled ? "Yes" : "No",
              },
              {
                label: "Deadline",
                key: "terms.deadline",
                children: new Date(contract.deadline).toLocaleString(),
              },

              ...(contract.reservation
                ? [
                    {
                      label: "Fund Amount",
                      key: "terms.reservedFundAmount",
                      children: (
                        <span>
                          <MoneyDisplay amount={contract.reservation.amount} />
                        </span>
                      ),
                    },
                    {
                      label: "Funds Spent",
                      key: "terms.reservedFundSpent",
                      children: (
                        <span>
                          <MoneyDisplay
                            amount={contract.reservation.actualAmount}
                          />
                        </span>
                      ),
                    },
                    {
                      label: "Fund Status",
                      key: "terms.reservedFundStatus",
                      children: (
                        <span>
                          {contract.reservation.status} (ID:{" "}
                          {contract.reservation.id})
                        </span>
                      ),
                    },
                  ]
                : []),
            ]}
          ></Descriptions>

          <Descriptions
            bordered
            size="small"
            column={2}
            // layout="vertical"
            items={[
              {
                label: "Payment on Accepted",
                key: "terms.payment.onAccepted",
                children: <MoneyDisplay amount={contract.onAccepted} />,
              },
              {
                label: "Travel (Fuel) Expense",
                key: "travel_expense",
                children: <MoneyDisplay amount={travelExpense} />,
              },
              {
                label: "Payment on Fulfilled",
                key: "terms.payment.onFulfilled",
                children: <MoneyDisplay amount={contract.onFulfilled} />,
              },
              {
                label: "Trade Unit Expense",
                key: "trade_unit_expense",
                children: <MoneyDisplay amount={tradeUnitExpense} />,
              },
              {
                label: "Total Reward",
                key: "total_reward",
                children: (
                  <MoneyDisplay
                    amount={
                      totalReward + contract.onAccepted + contract.onFulfilled
                    }
                  />
                ),
              },
              {
                label: "Total Expense",
                key: "total_expense",
                children: <MoneyDisplay amount={totalExpense} />,
              },
              {
                label: "Total Profit",
                key: "total_profit",
                children: <MoneyDisplay amount={totalProfit} />,
                span: 2,
              },
            ]}
          ></Descriptions>

          <Table
            size="small"
            loading={loading}
            columns={[
              {
                title: "Destination Symbol",
                dataIndex: "destinationSymbol",
                key: "destinationSymbol",
                render: (symbol: string) => (
                  <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
                ),
              },
              {
                title: "Trade Symbol",
                dataIndex: "tradeSymbol",
                key: "tradeSymbol",
                render: (value) => (
                  <Link to={`/supplyChain/${value}`}>{value}</Link>
                ),
              },
              {
                title: "Units Fulfilled",
                dataIndex: "unitsFulfilled",
                key: "unitsFulfilled",
              },
              {
                title: "Units Required",
                dataIndex: "unitsRequired",
                key: "unitsRequired",
              },
            ]}
            dataSource={contract.deliveries.items}
            rowKey={(record) =>
              "tt" +
              record.tradeSymbol +
              record.destinationSymbol +
              record.contractId +
              record.unitsRequired
            }
          ></Table>
        </Flex>
        <Table
          size="small"
          loading={loading}
          rowKey={(id) => id.id}
          columns={[
            {
              title: "ID",
              dataIndex: "id",
              key: "id",
            },
            {
              title: "Ship Symbol",
              dataIndex: "shipSymbol",
              key: "shipSymbol",
            },
            {
              title: "Trade Symbol",
              dataIndex: "tradeSymbol",
              key: "tradeSymbol",
              render: (value) => (
                <Link to={`/supplyChain/${value}`}>{value}</Link>
              ),
            },
            {
              title: "Units",
              dataIndex: "units",
              key: "units",
            },
            {
              title: "Destination Symbol",
              dataIndex: "destinationSymbol",
              key: "destinationSymbol",
              render: (symbol: string) => (
                <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
              ),
            },
            {
              title: "Purchase Symbol",
              dataIndex: "purchaseSymbol",
              key: "purchaseSymbol",
              render: (symbol: string) => (
                <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
              ),
            },
            {
              title: "Created At",
              dataIndex: "createdAt",
              key: "createdAt",
              render: (date: string) => new Date(date).toLocaleString(),
            },
            {
              title: "Updated At",
              dataIndex: "updatedAt",
              key: "updatedAt",
              render: (date: string) => new Date(date).toLocaleString(),
            },
            {
              title: "Status",
              dataIndex: "status",
              key: "status",
            },
          ]}
          dataSource={contract.shipments.items}
        ></Table>
        <TransactionTable
          transactions={contract.marketTransactions.items}
          reasons={{
            contract: true,
            trade_route_id: false,
            mining: false,
            construction_shipment_id: false,
          }}
        />
      </Flex>
    </div>
  );
}

export default Contract;
