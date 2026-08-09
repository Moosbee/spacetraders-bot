import { useQuery } from "@apollo/client/react";
import { Button, Space, Table, TableProps } from "antd";
import { useMemo } from "react";
import PageTitle from "../features/PageTitle";
import { GET_ALL_SHIP_ROUTES } from "../graphql/queries";

function getInterSystemTravelStats(
  engineSpeed: number,
  flightMode: "BURN" | "CRUISE" | "STEALTH" | "DRIFT",
  distance: number,
  engineCondition: number = 1,
) {
  let fuelCost = 1;
  let multiplier = 1;

  switch (flightMode) {
    case "BURN":
      fuelCost = Math.max(2, 2 * Math.round(distance));
      multiplier = 12.5;
      break;
    case "CRUISE":
      fuelCost = Math.max(1, Math.round(distance));
      multiplier = 25;
      break;
    case "STEALTH": // Same logic for "CRUISE" and "STEALTH"
      fuelCost = Math.max(1, Math.round(distance));
      multiplier = 30;
      break;
    case "DRIFT":
      fuelCost = 1;
      multiplier = 250;
      break;
  }

  const conditionMuliplier = 1;
  // const conditionMuliplier = 2 - 0.5 - 1 / (engineCondition + 1);
  // const conditionMuliplier = 1 / (-engineCondition + 2);
  // const conditionMuliplier = engineCondition * 0.5 + 0.5;
  // const conditionMuliplier = 0.5 * engineCondition * engineCondition + 0.5; // best so far
  // const conditionMuliplier = 0.6 * engineCondition * engineCondition + 0.4; // not just overshooting but also undershooting

  const travelTime = Math.round(
    Math.max(1, Math.round(distance)) *
      (multiplier / (engineSpeed * conditionMuliplier)) +
      15,
  );

  return {
    calcFuelCost: fuelCost,
    calcTravelTime: travelTime,
  };
}

function ShipRoutes() {
  const { loading, error, data, dataState, refetch } =
    useQuery(GET_ALL_SHIP_ROUTES);

  const routes = useMemo(
    () =>
      data?.shipRoutes?.items.map((route) => ({
        ...route,
        travelTime: Math.round(route.travelTime),
        ...getInterSystemTravelStats(
          route.shipStateBefore?.engineSpeed || 1,
          route.navMode as "BURN" | "CRUISE" | "STEALTH" | "DRIFT",
          route.distance,
          route.shipStateBefore?.engineCondition,
        ),
      })) ?? [],
    [data?.shipRoutes?.items],
  );

  if (dataState != "complete") return <p>Loading...</p>;
  if (error) return <p>Error: {error.message}</p>;

  const columns: TableProps<(typeof routes)[number]>["columns"] = [
    // {
    //   title: "ID",
    //   dataIndex: "id",
    //   key: "id",
    //   sorter: (a, b) => a.id - b.id,
    //   defaultSortOrder: "descend",
    // },
    // {
    //   title: "From",
    //   dataIndex: "from",
    //   key: "from",
    //   render: (symbol: string) => (
    //     <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
    //   ),
    //   sorter: (a, b) => a.from.localeCompare(b.from),
    // },
    // {
    //   title: "To",
    //   dataIndex: "to",
    //   key: "to",
    //   render: (symbol: string) => (
    //     <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
    //   ),
    //   sorter: (a, b) => a.to.localeCompare(b.to),
    // },
    // {
    //   title: "Ship Symbol",
    //   dataIndex: "shipSymbol",
    //   key: "shipSymbol",
    //   render: (symbol: string) => <Link to={`/ships/${symbol}`}>{symbol}</Link>,
    //   sorter: (a, b) => a.shipSymbol.localeCompare(b.shipSymbol),
    // },
    {
      title: "Nav Mode",
      dataIndex: "navMode",
      key: "navMode",
      sorter: {
        compare: (a, b) => a.navMode.localeCompare(b.navMode),
      },
      filters: [
        {
          text: "BURN",
          value: "BURN",
        },
        {
          text: "CRUISE",
          value: "CRUISE",
        },
        {
          text: "DRIFT",
          value: "DRIFT",
        },
        {
          text: "STEALTH",
          value: "STEALTH",
        },
      ],
      onFilter: (value, record) => record.navMode === value,
    },
    {
      title: "Distance",
      dataIndex: "distance",
      key: "distance",
      sorter: {
        compare: (a, b) => a.distance - b.distance,
        multiple: 4,
      },
    },
    {
      title: "Engine Speed",
      key: "shipStateBefore.engineSpeed",
      render: (_, record) => record.shipStateBefore?.engineSpeed,
      sorter: {
        compare: (a, b) =>
          (a.shipStateBefore?.engineSpeed || 0) -
          (b.shipStateBefore?.engineSpeed || 0),
        multiple: 3,
      },
    },
    {
      title: "Calc Travel Time",
      dataIndex: "calcTravelTime",
      key: "calcTravelTime",
      sorter: (a, b) => a.calcTravelTime - b.calcTravelTime,
    },
    {
      title: "Real Travel Time",
      dataIndex: "travelTime",
      key: "travelTime",
      sorter: (a, b) => a.travelTime - b.travelTime,
    },
    {
      title: "Time Diff",
      key: "timeDiff",
      render: (_, record) => record.travelTime - record.calcTravelTime,
      sorter: (a, b) =>
        a.travelTime - a.calcTravelTime - (b.travelTime - b.calcTravelTime),
    },
    {
      title: "Time Diff %",
      key: "timeDiffPercent",
      render: (_, record) =>
        ((record.travelTime - record.calcTravelTime) / record.calcTravelTime) *
        100,
      sorter: (a, b) =>
        (a.travelTime - a.calcTravelTime) / a.calcTravelTime -
        (b.travelTime - b.calcTravelTime) / b.calcTravelTime,
    },
    // {
    //   title: "Calc Fuel Cost",
    //   dataIndex: "calcFuelCost",
    //   key: "calcFuelCost",
    //   sorter: (a, b) => a.calcFuelCost - b.calcFuelCost,
    // },
    // {
    //   title: "Real Fuel Cost",
    //   dataIndex: "fuelCost",
    //   key: "fuelCost",
    //   sorter: (a, b) => a.fuelCost - b.fuelCost,
    // },
    {
      title: "Engine C B",
      key: "shipStateBefore.engineCondition",
      render: (_, record) => record.shipStateBefore?.engineCondition,
      sorter: {
        compare: (a, b) =>
          (a.shipStateBefore?.engineCondition || 0) -
          (b.shipStateBefore?.engineCondition || 0),
        multiple: 2,
      },
    },
    {
      title: "Frame C B",
      key: "shipStateBefore.frameCondition",
      render: (_, record) => record.shipStateBefore?.frameCondition,
      sorter: (a, b) =>
        (a.shipStateBefore?.frameCondition || 0) -
        (b.shipStateBefore?.frameCondition || 0),
    },
    {
      title: "Reactor C B",
      key: "shipStateBefore.reactorCondition",
      render: (_, record) => record.shipStateBefore?.reactorCondition,
      sorter: (a, b) =>
        (a.shipStateBefore?.reactorCondition || 0) -
        (b.shipStateBefore?.reactorCondition || 0),
    },
    {
      title: "Engine C A",
      key: "shipStateAfter.engineCondition",
      render: (_, record) => record.shipStateAfter?.engineCondition,
      sorter: (a, b) =>
        (a.shipStateAfter?.engineCondition || 0) -
        (b.shipStateAfter?.engineCondition || 0),
    },
    {
      title: "Frame C A",
      key: "shipStateAfter.frameCondition",
      render: (_, record) => record.shipStateAfter?.frameCondition,
      sorter: (a, b) =>
        (a.shipStateAfter?.frameCondition || 0) -
        (b.shipStateAfter?.frameCondition || 0),
    },
    {
      title: "Reactor C A",
      key: "shipStateAfter.reactorCondition",
      render: (_, record) => record.shipStateAfter?.reactorCondition,
      sorter: (a, b) =>
        (a.shipStateAfter?.reactorCondition || 0) -
        (b.shipStateAfter?.reactorCondition || 0),
    },
    {
      title: "Incident",
      key: "incident",
      render: (_, record) =>
        record.shipStateBefore?.engineCondition !==
          record.shipStateAfter?.engineCondition ||
        record.shipStateBefore?.frameCondition !==
          record.shipStateAfter?.frameCondition ||
        record.shipStateBefore?.reactorCondition !==
          record.shipStateAfter?.reactorCondition ||
        record.shipStateBefore?.engineSpeed !==
          record.shipStateAfter?.engineSpeed
          ? "yes"
          : "no",
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="Ship Routes" />
      <Space>
        <h1 className="scroll-m-20 text-center text-3xl font-bold tracking-tight text-balance">
          Ship Routes {data.shipRoutes?.items.length}
        </h1>
        <Button
          onClick={() => {
            refetch();
          }}
        >
          Refresh
        </Button>
        <Button
          onClick={() => {
            const a = document.createElement("a");
            const routeText = routes
              .filter(
                (record) =>
                  !(
                    record.shipStateBefore?.engineCondition !==
                      record.shipStateAfter?.engineCondition ||
                    record.shipStateBefore?.frameCondition !==
                      record.shipStateAfter?.frameCondition ||
                    record.shipStateBefore?.reactorCondition !==
                      record.shipStateAfter?.reactorCondition ||
                    record.shipStateBefore?.engineSpeed !==
                      record.shipStateAfter?.engineSpeed
                  ),
              )
              .map(
                (r) =>
                  `${r.shipStateBefore?.engineSpeed}; ${r.navMode}; ${r.travelTime}; ${r.calcTravelTime}; ${r.shipStateBefore?.engineCondition};`,
              )
              .join("\n");
            const text = `EngineSpeed; NavMode; TravelTime; CalcTravelTime; EngineCondition;\n${routeText}`;
            const type = "text/csv;charset=utf-8;";
            const name = "routes.csv";
            const file = new Blob([text], { type: type });
            a.href = URL.createObjectURL(file);
            a.download = name;
            a.click();
          }}
        >
          Export
        </Button>
      </Space>
      <Table
        dataSource={routes}
        columns={columns}
        loading={loading}
        rowKey="id"
        pagination={{
          showSizeChanger: true,
          pageSizeOptions: [
            "10",
            "20",
            "50",
            "100",
            "200",
            "500",
            "1000",
            "10000",
          ],
          defaultPageSize: 100,
          showTotal: (total, range) => `${range[0]}-${range[1]} of ${total}`,
        }}
      />
    </div>
  );
}

export default ShipRoutes;
