import { Button, Descriptions, Flex, Space, Table } from "antd";
import { useEffect, useMemo, useState } from "react";
import { useParams } from "react-router-dom";
import { backendUrl } from "../data";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import TransactionTable from "../features/TransactionTable/TransactionTable";
import WaypointLink from "../features/WaypointLink";
import { ContractDeliverable, ContractResponse } from "../models/SQLContract";

function Contract() {
  const { contractID } = useParams();

  const [contractResp, setContract] = useState<ContractResponse | null>(null);

  useEffect(() => {
    fetch(`http://${backendUrl}/contracts/${contractID}`)
      .then((response) => response.json())
      .then((data) => {
        console.log("Contract", data);

        setContract(data);
      });
  }, [contractID]);

  const { total_expense, total_reward } = useMemo(() => {
    if (!contractResp) {
      return { total_expense: 0, total_reward: 0 };
    }

    const erg = contractResp[3].reduce(
      (a, b) => {
        if (b.type === "PURCHASE") {
          a.total_expense += b.total_price;
        }

        if (b.type === "SELL") {
          a.total_reward += b.total_price;
        }

        return a;
      },
      { total_expense: 0, total_reward: 0 }
    );

    return erg;
  }, [contractResp]);

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title={`Contract ${contractID}`} />
      <Space>
        <h1>Contract {contractID}</h1>
        <Button
          onClick={() => {
            fetch(`http://${backendUrl}/contracts/${contractID}`)
              .then((response) => response.json())
              .then((data) => {
                console.log("Contract", data);

                setContract(data);
              });
          }}
        >
          Reload
        </Button>
      </Space>
      {contractResp && (
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
                  children: contractResp[1].contract_type,
                },
                {
                  label: "Accepted",
                  key: "accepted",
                  children: contractResp[1].accepted ? "Yes" : "No",
                },
                {
                  label: "deadlineToAccept",
                  key: "deadlineToAccept",
                  children: new Date(
                    contractResp[1].deadline_to_accept
                  ).toLocaleString(),
                },

                {
                  label: "Faction Symbol",
                  key: "factionSymbol",
                  children: contractResp[1].faction_symbol,
                },
                {
                  label: "Fulfilled",
                  key: "fulfilled",
                  children: contractResp[1].fulfilled ? "Yes" : "No",
                },
                {
                  label: "Deadline",
                  key: "terms.deadline",
                  children: new Date(contractResp[1].deadline).toLocaleString(),
                },

                ...(contractResp[5]
                  ? [
                      {
                        label: "Fund Amount",
                        key: "terms.reservedFundAmount",
                        children: (
                          <span>
                            <MoneyDisplay amount={contractResp[5]?.amount} />
                          </span>
                        ),
                      },
                      {
                        label: "Funds Spent",
                        key: "terms.reservedFundSpent",
                        children: (
                          <span>
                            <MoneyDisplay
                              amount={contractResp[5]?.actual_amount}
                            />
                          </span>
                        ),
                      },
                      {
                        label: "Fund Status",
                        key: "terms.reservedFundStatus",
                        children: (
                          <span>
                            {contractResp[5]?.status} (ID: {contractResp[5]?.id}
                            )
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
              column={1}
              // layout="vertical"
              items={[
                {
                  label: "Payment on Accepted",
                  key: "terms.payment.onAccepted",
                  children: (
                    <MoneyDisplay amount={contractResp[1].on_accepted} />
                  ),
                },
                {
                  label: "Payment on Fulfilled",
                  key: "terms.payment.onFulfilled",
                  children: (
                    <MoneyDisplay amount={contractResp[1].on_fulfilled} />
                  ),
                },
                {
                  label: "Total Reward",
                  key: "total_reward",
                  children: (
                    <MoneyDisplay
                      amount={
                        total_reward +
                        contractResp[1].on_accepted +
                        contractResp[1].on_fulfilled
                      }
                    />
                  ),
                },
                {
                  label: "Total Expense",
                  key: "total_expense",
                  children: <MoneyDisplay amount={total_expense} />,
                },
                {
                  label: "Total Profit",
                  key: "total_profit",
                  children: (
                    <MoneyDisplay
                      amount={
                        total_reward +
                        contractResp[1].on_accepted +
                        contractResp[1].on_fulfilled -
                        total_expense
                      }
                    />
                  ),
                },
              ]}
            ></Descriptions>

            <Table
              size="small"
              columns={[
                {
                  title: "Destination Symbol",
                  dataIndex: "destination_symbol",
                  key: "destination_symbol",
                  render: (symbol: string) => (
                    <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
                  ),
                },
                {
                  title: "Trade Symbol",
                  dataIndex: "trade_symbol",
                  key: "trade_symbol",
                },
                {
                  title: "Units Fulfilled",
                  dataIndex: "units_fulfilled",
                  key: "units_fulfilled",
                },
                {
                  title: "Units Required",
                  dataIndex: "units_required",
                  key: "units_required",
                },
              ]}
              dataSource={contractResp[2]}
              rowKey={(record: ContractDeliverable) =>
                "tt" +
                record.trade_symbol +
                record.destination_symbol +
                record.contract_id +
                record.units_required
              }
            ></Table>
          </Flex>
          <Table
            size="small"
            rowKey={(id) => id.id}
            columns={[
              {
                title: "ID",
                dataIndex: "id",
                key: "id",
              },
              {
                title: "Ship Symbol",
                dataIndex: "ship_symbol",
                key: "ship_symbol",
              },
              {
                title: "Trade Symbol",
                dataIndex: "trade_symbol",
                key: "trade_symbol",
              },
              {
                title: "Units",
                dataIndex: "units",
                key: "units",
              },
              {
                title: "Destination Symbol",
                dataIndex: "destination_symbol",
                key: "destination_symbol",
                render: (symbol) => (
                  <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
                ),
              },
              {
                title: "Purchase Symbol",
                dataIndex: "purchase_symbol",
                key: "purchase_symbol",
                render: (symbol) => (
                  <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
                ),
              },
              {
                title: "Created At",
                dataIndex: "created_at",
                key: "created_at",
                render: (date) => new Date(date).toLocaleString(),
              },
              {
                title: "Updated At",
                dataIndex: "updated_at",
                key: "updated_at",
                render: (date) => new Date(date).toLocaleString(),
              },
              {
                title: "Status",
                dataIndex: "status",
                key: "status",
              },
            ]}
            dataSource={contractResp[4]}
          ></Table>
          <TransactionTable
            transactions={contractResp[3]}
            reasons={{ contract: true, trade_route: false, mining: false }}
          />
        </Flex>
      )}
    </div>
  );
}

export default Contract;
