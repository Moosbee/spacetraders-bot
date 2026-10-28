import { Button, List, Table, TableProps, Tooltip } from "antd";
import {
  ActivityLevel,
  GetWaypointQuery,
  ShipEngineSymbol,
  ShipFrameSymbol,
  ShipModuleSymbol,
  ShipMountSymbol,
  ShipReactorSymbol,
  ShipType,
  SupplyLevel,
} from "../../gql/graphql";
import MoneyDisplay from "../MonyDisplay";

type ShipyardShip =
  GetWaypointQuery["waypoint"]["shipyardShips"]["items"][number];

function ShipyardShipTable({ ships }: { ships: ShipyardShip[] }) {
  const columns: TableProps<ShipyardShip>["columns"] = [
    {
      title: "Ship Type",
      dataIndex: "shipType",
      key: "shipType",
      sorter: (a, b) => a.shipType.localeCompare(b.shipType),
      filters: Object.values(ShipType).map((shipType) => ({
        text: shipType,
        value: shipType,
      })),
      onFilter: (value, record) => record.shipType === value,
    },
    {
      title: "Action",
      dataIndex: "",
      key: "x",
      render: () => <Button disabled>Purchase</Button>,
    },
    {
      title: "Purchase Price",
      dataIndex: "purchasePrice",
      key: "purchasePrice",
      align: "right",
      render: (price: number) => <MoneyDisplay amount={price} />,
      sorter: (a, b) => a.purchasePrice - b.purchasePrice,
    },
    {
      title: "Supply Level",
      dataIndex: "supply",
      key: "supply",
      filters: Object.values(SupplyLevel).map((supply) => ({
        text: supply,
        value: supply,
      })),
      onFilter: (value, record) => record.supply === value,
      sorter: (a, b) => a.supply.localeCompare(b.supply),
    },
    {
      title: "Activity",
      dataIndex: "activity",
      key: "activity",
      sorter: (a, b) => a.activity?.localeCompare(b.activity ?? "") ?? 0,
      filters: Object.values(ActivityLevel).map((activity) => ({
        text: activity,
        value: activity,
      })),
    },
    {
      title: "Engine Type",
      key: "engine_type",
      render: (_, record) => (
        <Tooltip
          title={`Quality: ${record.engineQuality}\nSpeed: ${record.engineInfo.speed}`}
        >
          {record.engineInfo.symbol} ({record.engineInfo.speed})
        </Tooltip>
      ),
      sorter: (a, b) => a.engineInfo.symbol.localeCompare(b.engineInfo.symbol),
      filters: Object.values(ShipEngineSymbol).map((engine) => ({
        text: engine,
        value: engine,
      })),
      onFilter: (value, record) => record.engineInfo.symbol === value,
    },
    {
      title: "Frame Type",
      key: "frame_type",
      render: (_, record) => (
        <Tooltip
          title={`Quality: ${record.frameQuality}\nFuel: ${record.frameInfo.fuelCapacity}\nModules: ${record.frameInfo.moduleSlots}\nMounts: ${record.frameInfo.mountingPoints}`}
        >
          {record.frameInfo.symbol} ({record.frameInfo.fuelCapacity})
        </Tooltip>
      ),
      sorter: (a, b) => a.frameInfo.symbol.localeCompare(b.frameInfo.symbol),
      filters: Object.values(ShipFrameSymbol).map((frame) => ({
        text: frame,
        value: frame,
      })),

      onFilter: (value, record) => record.frameInfo.symbol === value,
    },
    {
      title: "Reactor Type",
      dataIndex: "reactor_type",
      key: "reactor_type",
      render: (_, record) => (
        <Tooltip
          title={`Quality: ${record.reactorQuality}\nPower Output: ${record.reactorInfo.powerOutput}`}
        >
          {record.reactorInfo.symbol} ({record.reactorInfo.powerOutput})
        </Tooltip>
      ),
      sorter: (a, b) =>
        a.reactorInfo.symbol.localeCompare(b.reactorInfo.symbol),
      filters: Object.values(ShipReactorSymbol).map((reactor) => ({
        text: reactor,
        value: reactor,
      })),
      onFilter: (value, record) => record.reactorInfo.symbol === value,
    },
    {
      title: "Modules",
      dataIndex: "modules",
      key: "modules",
      render: (modules: ShipModuleSymbol[]) =>
        modules.length > 0 ? (
          <List
            size="small"
            dataSource={modules}
            renderItem={(item) => <List.Item>{item}</List.Item>}
          />
        ) : (
          "No modules"
        ),
    },
    {
      title: "Mounts",
      dataIndex: "mounts",
      key: "mounts",
      render: (mounts: ShipMountSymbol[]) =>
        mounts.length > 0 ? (
          <List
            size="small"
            dataSource={mounts}
            renderItem={(item) => <List.Item>{item}</List.Item>}
          />
        ) : (
          "No mounts"
        ),
    },
    {
      title: "Created At",
      dataIndex: "createdAt",
      key: "createdAt",
      render: (date: string) => <span>{new Date(date).toLocaleString()}</span>,
    },
  ];

  return <Table dataSource={ships} columns={columns} size="small" />;
}

export default ShipyardShipTable;
