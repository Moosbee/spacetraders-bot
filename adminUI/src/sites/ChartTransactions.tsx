import { useQuery } from "@apollo/client/react";
import { Button, Col, Divider, Flex, Popover, Row, Space, Table } from "antd";
import { useCallback, useState } from "react";
import { Link } from "react-router-dom";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import WaypointLink from "../features/WaypointLink";
import { WaypointTraitSymbol, WaypointType } from "../gql/graphql";
import { GET_CHART_TRANSACTIONS } from "../graphql/queries";

function ChartTransactions() {
  const { loading, error, data, refetch } = useQuery(
    GET_CHART_TRANSACTIONS
    // { pollInterval: 3600000 }
  );

  const [traitToPriceMap, setTraitToPriceMap] = useState(
    Object.values(WaypointTraitSymbol).reduce((val, type) => {
      val[type] = 0;
      return val;
    }, {} as Record<WaypointTraitSymbol, number>)
  );

  const calculatePrices = useCallback(() => {
    const newTraitToPriceMap: Record<WaypointTraitSymbol, number> = {
      ...traitToPriceMap,
    };

    /*
    assumption:
    Each trait have a set amount that will be added to the chart reward, this reward stays constant
    Traits may appear twice on a waypoint, some traits are not used
    The price is calculated like this(pseudocode)
    totalReward=5000+waypoint.traits.map((trait)=>traitToPriceMap[trait]).sum()
  */

    const BASE_PRICE = 5000;

    // Collect all unique traits
    const traitsSet = new Set<WaypointTraitSymbol>();
    data?.chartTransactions.forEach((t) => {
      if (t.waypoint) {
        t.waypoint.traits.forEach((trait) => traitsSet.add(trait));
      }
    });

    const traitsList = Array.from(traitsSet);
    const numTraits = traitsList.length;

    // Build coefficient matrix A and result vector b
    const A: number[][] = [];
    const b: number[] = [];

    data?.chartTransactions.forEach((transaction) => {
      if (!transaction.waypoint) return;

      // Create a row with coefficients for each trait
      const row = new Array(numTraits).fill(0);

      // Count occurrences of each trait (handles duplicates)
      transaction.waypoint.traits.forEach((trait) => {
        const traitIdx = traitsList.indexOf(trait);
        row[traitIdx] += 1; // Increment if trait appears multiple times
      });

      A.push(row);
      // Subtract base price from total
      b.push(transaction.totalPrice - BASE_PRICE);
    });

    // Solve using least squares
    const solution = solveLinearSystem(A, b);

    // Map back to trait names
    const traitPrices = new Map<WaypointTraitSymbol, number>();
    traitsList.forEach((trait, i) => {
      traitPrices.set(trait, solution[i]);
    });

    traitsList.forEach((trait) => {
      newTraitToPriceMap[trait] = traitPrices.get(trait) || 0;
    });

    setTraitToPriceMap(newTraitToPriceMap);
  }, [data, traitToPriceMap]);

  if (error) return <p>Error: {error.message}</p>;

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle
        title={`Chart Transactions ${data?.chartTransactions.length}`}
      />
      <Space>
        <h1>Chart Transactions {data?.chartTransactions.length}</h1>
        <Button
          onClick={() => {
            refetch();
          }}
        >
          Refresh
        </Button>
        <Button
          onClick={() => {
            calculatePrices();
          }}
        >
          Calculate Prices
        </Button>
      </Space>
      <Divider />
      <Row gutter={10}>
        <Col>
          <Table
            rowKey={(id) => id.waypointSymbol}
            loading={loading}
            dataSource={data?.chartTransactions || []}
            columns={[
              {
                title: "Waypoint",
                dataIndex: "waypointSymbol",
                key: "waypointSymbol",
                render: (symbol: string | undefined) =>
                  symbol ? (
                    <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
                  ) : (
                    "N/A"
                  ),
                sorter: (a, b) =>
                  (a.waypointSymbol || "").localeCompare(
                    b.waypointSymbol || ""
                  ),

                filters: [
                  ...new Set(
                    (data?.chartTransactions || []).map(
                      (t) => t.waypointSymbol || ""
                    )
                  ),
                ].map((t) => ({
                  text: t,
                  value: t,
                })),
                onFilter: (value, record) => record.waypointSymbol === value,
              },
              {
                title: "Ship",
                dataIndex: "shipSymbol",
                key: "shipSymbol",
                render: (symbol: string) => (
                  <Link to={`/ships/${symbol}`}>{symbol}</Link>
                ),
                sorter: (a, b) =>
                  (a.shipSymbol || "").localeCompare(b.shipSymbol || ""),
                filters: [
                  ...new Set(
                    (data?.chartTransactions || []).map(
                      (t) => t.shipSymbol || ""
                    )
                  ),
                ].map((t) => ({
                  text: t,
                  value: t,
                })),
                onFilter: (value, record) => record.shipSymbol === value,
              },
              {
                title: "Waypoint Type",
                key: "waypointType",
                render: (_, record) => record.waypoint?.waypointType,
                sorter: (a, b) =>
                  (a.waypoint?.waypointType ?? "").localeCompare(
                    b.waypoint?.waypointType ?? ""
                  ),
                filters: Object.values(WaypointType).map((t) => ({
                  text: t,
                  value: t,
                })),
                onFilter: (value, record) =>
                  record.waypoint?.waypointType === value,
              },
              {
                title: "Waypoint Traits",
                key: "waypointTraits",
                align: "end",
                render: (_, record) => (
                  <Popover
                    title={
                      <Flex vertical>
                        {record.waypoint?.traits.map((t) => (
                          <span>{t}</span>
                        ))}
                      </Flex>
                    }
                  >
                    {record.waypoint?.traits.length}
                  </Popover>
                ),
                sorter: (a, b) =>
                  (a.waypoint?.traits.length ?? 0) -
                  (b.waypoint?.traits.length ?? 0),
                filters: Object.values(WaypointTraitSymbol).map((t) => ({
                  text: t,
                  value: t,
                })),
                onFilter: (value, record) =>
                  record.waypoint?.traits.some((t) => {
                    return t == value;
                  }) || false,
              },
              {
                title: "Total Price",
                dataIndex: "totalPrice",
                key: "totalPrice",
                render: (value) => <MoneyDisplay amount={value} />,
                align: "right",
                sorter: (a, b) => (a.totalPrice ?? 0) - (b.totalPrice ?? 0),
              },
              {
                title: "Timestamp",
                dataIndex: "timestamp",
                key: "timestamp",
                render: (value) => new Date(value).toLocaleString(),
                align: "right",
                sorter: (a, b) =>
                  new Date(a.timestamp ?? 0).getTime() -
                  new Date(b.timestamp ?? 0).getTime(),
                defaultSortOrder: "descend",
              },
            ]}
            pagination={{
              showSizeChanger: true,
              pageSizeOptions: ["10", "20", "50", "100", "200", "500", "1000"],
              defaultPageSize: 10,
              showTotal: (total, range) =>
                `${range[0]}-${range[1]} of ${total}`,
            }}
          />
        </Col>
        <Col>
          <Table
            loading={loading}
            title={() => "Trait Prices"}
            dataSource={Object.entries(traitToPriceMap).map((value) => {
              return {
                type: value[0],
                amount: value[1],
              };
            })}
            columns={[
              {
                title: "Trait",
                dataIndex: "type",
                key: "type",
                sorter: (a, b) => (a.type || "").localeCompare(b.type || ""),
                filters: Object.values(WaypointTraitSymbol)
                  .sort((a, b) => a.localeCompare(b))
                  .map((type) => ({
                    text: type,
                    value: type,
                  })),
                onFilter: (value, record) => record.type === value,
              },
              {
                title: "Total Price",
                dataIndex: "amount",
                key: "amount",
                render: (value) => <MoneyDisplay amount={value} />,
                align: "right",
                sorter: (a, b) => (a.amount ?? 0) - (b.amount ?? 0),
              },
            ]}
            pagination={{
              showSizeChanger: true,
              pageSizeOptions: ["10", "20", "50", "100", "200", "500", "1000"],
              defaultPageSize: 10,
              showTotal: (total, range) =>
                `${range[0]}-${range[1]} of ${total}`,
            }}
          />
        </Col>
      </Row>
    </div>
  );
}

export default ChartTransactions;
function solveLinearSystem(A: number[][], b: number[]) {
  // least-squares via normal equations with small Tikhonov regularization
  const m = A.length;
  const n = m > 0 ? A[0].length : 0;
  if (n === 0) return [];

  // transpose A
  const AT: number[][] = Array.from({ length: n }, () => Array(m).fill(0));
  for (let i = 0; i < m; i++) {
    for (let j = 0; j < n; j++) {
      AT[j][i] = A[i][j];
    }
  }

  // compute ATA = AT * A (n x n) and ATb = AT * b (n)
  const ATA: number[][] = Array.from({ length: n }, () => Array(n).fill(0));
  const ATb: number[] = Array(n).fill(0);
  for (let i = 0; i < n; i++) {
    for (let k = 0; k < m; k++) {
      const aik = AT[i][k]; // equals A[k][i]
      ATb[i] += aik * b[k];
      for (let j = 0; j < n; j++) {
        ATA[i][j] += aik * A[k][j];
      }
    }
  }

  // regularize diagonal to improve conditioning
  const reg = 1e-8;
  for (let i = 0; i < n; i++) ATA[i][i] += reg;

  // solve ATA * x = ATb via Gaussian elimination with partial pivoting
  // build augmented matrix
  const aug: number[][] = ATA.map((row, i) => [...row, ATb[i]]);

  for (let col = 0; col < n; col++) {
    // pivot
    let pivotRow = col;
    let maxAbs = Math.abs(aug[col][col]);
    for (let r = col + 1; r < n; r++) {
      const val = Math.abs(aug[r][col]);
      if (val > maxAbs) {
        maxAbs = val;
        pivotRow = r;
      }
    }
    if (pivotRow !== col) {
      const tmp = aug[col];
      aug[col] = aug[pivotRow];
      aug[pivotRow] = tmp;
    }

    // if pivot is (near) zero, continue (regularization above should avoid this)
    const pivot = aug[col][col];
    if (Math.abs(pivot) < 1e-12) {
      // try to perturb slightly
      aug[col][col] = pivot + 1e-12;
    }

    // normalize and eliminate
    for (let r = col + 1; r < n; r++) {
      const factor = aug[r][col] / aug[col][col];
      if (factor === 0) continue;
      for (let c = col; c <= n; c++) {
        aug[r][c] -= factor * aug[col][c];
      }
    }
  }

  // back substitution
  const x = Array(n).fill(0);
  for (let i = n - 1; i >= 0; i--) {
    let sum = aug[i][n];
    for (let j = i + 1; j < n; j++) sum -= aug[i][j] * x[j];
    const diag = aug[i][i];
    x[i] = Math.abs(diag) < 1e-12 ? 0 : sum / diag;
  }

  return x;
}
