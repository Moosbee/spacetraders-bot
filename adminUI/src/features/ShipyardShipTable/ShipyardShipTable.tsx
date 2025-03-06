import { Button, List, Table, TableProps, Tooltip } from "antd";
import { ShipyardShip } from "../../models/Shipyard";
import {
  ActivityLevel,
  ShipEngineSymbolEnum,
  ShipModuleSymbolEnum,
  ShipMountSymbolEnum,
  ShipType,
  SupplyLevel,
} from "../../models/api";
import MoneyDisplay from "../MonyDisplay";

function ShipyardShipTable({
  ships,
  onPurchase,
}: {
  ships: ShipyardShip[];
  onPurchase: (ship: ShipyardShip) => void;
}) {
  const columns: TableProps<ShipyardShip>["columns"] = [
    {
      title: "Ship Type",
      dataIndex: "ship_type",
      key: "ship_type",
      sorter: (a, b) => a.ship_type.localeCompare(b.ship_type),
      filters: Object.values(ShipType).map((ship_type) => ({
        text: ship_type,
        value: ship_type,
      })),
      onFilter: (value, record) => record.ship_type === value,
    },
    {
      title: "Action",
      dataIndex: "",
      key: "x",
      render: (_value, record) => (
        <Button onClick={() => onPurchase(record)}>Purchase</Button>
      ),
    },
    {
      title: "Purchase Price",
      dataIndex: "purchase_price",
      key: "purchase_price",
      align: "right",
      render: (price: number) => <MoneyDisplay amount={price} />,
      sorter: (a, b) => a.purchase_price - b.purchase_price,
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
      sorter: (a, b) => a.activity.localeCompare(b.activity),
      filters: Object.values(ActivityLevel).map((activity) => ({
        text: activity,
        value: activity,
      })),
    },
    {
      title: "Crew",
      dataIndex: "crew_capacity",
      key: "crew_capacity",
      render: (_crew: number, record) =>
        `${record.crew_requirement} / ${record.crew_capacity}`,
    },
    {
      title: "Engine Type",
      dataIndex: "engine_type",
      key: "engine_type",
      render: (engine: ShipEngineSymbolEnum, record) => (
        <Tooltip title={`Quality: ${record.engine_quality}`}>{engine}</Tooltip>
      ),
      sorter: (a, b) => a.engine_type.localeCompare(b.engine_type),
      filters: Object.values(ShipEngineSymbolEnum).map((engine) => ({
        text: engine,
        value: engine,
      })),
    },
    {
      title: "Frame Type",
      dataIndex: "frame_type",
      key: "frame_type",
      render: (frame: ShipEngineSymbolEnum, record) => (
        <Tooltip title={`Quality: ${record.frame_quality}`}>{frame}</Tooltip>
      ),
      sorter: (a, b) => a.frame_type.localeCompare(b.frame_type),
      filters: Object.values(ShipEngineSymbolEnum).map((frame) => ({
        text: frame,
        value: frame,
      })),

      onFilter: (value, record) => record.frame_type === value,
    },
    {
      title: "Reactor Type",
      dataIndex: "reactor_type",
      key: "reactor_type",
      render: (reactor: ShipEngineSymbolEnum, record) => (
        <Tooltip title={`Quality: ${record.reactor_quality}`}>
          {reactor}
        </Tooltip>
      ),
      sorter: (a, b) => a.reactor_type.localeCompare(b.reactor_type),
      filters: Object.values(ShipEngineSymbolEnum).map((reactor) => ({
        text: reactor,
        value: reactor,
      })),
      onFilter: (value, record) => record.reactor_type === value,
    },
    {
      title: "Modules",
      dataIndex: "modules",
      key: "modules",
      render: (modules: ShipModuleSymbolEnum[]) => (
        <List
          size="small"
          dataSource={modules}
          renderItem={(item) => <List.Item>{item}</List.Item>}
        />
      ),
    },
    {
      title: "Mounts",
      dataIndex: "mounts",
      key: "mounts",
      render: (mounts: ShipMountSymbolEnum[]) => (
        <List
          size="small"
          dataSource={mounts}
          renderItem={(item) => <List.Item>{item}</List.Item>}
        />
      ),
    },
    {
      title: "Created At",
      dataIndex: "created_at",
      key: "created_at",
      render: (date: string) => <span>{new Date(date).toLocaleString()}</span>,
    },
  ];

  return <Table dataSource={ships} columns={columns} size="small" />;
}

export default ShipyardShipTable;
