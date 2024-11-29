import { Table } from "antd";
import { useEffect } from "react";
import PageTitle from "../features/PageTitle";
import useMyStore, { backendUrl } from "../store";

function Ships() {
  const ships = useMyStore((state) => state.ships);
  const setShips = useMyStore((state) => state.setShips);

  useEffect(() => {
    fetch(`http://${backendUrl}/ships`)
      .then((response) => response.json())
      .then(setShips);
  }, [setShips]);

  const columns = [
    {
      title: "Symbol",
      dataIndex: "symbol",
      key: "symbol",
    },
    {
      title: "Role",
      dataIndex: "role",
      key: "role",
    },
    {
      title: "Registration Role",
      dataIndex: "registration_role",
      key: "registration_role",
    },
    {
      title: "Engine Speed",
      dataIndex: "engine_speed",
      key: "engine_speed",
    },
    {
      title: "Current System",
      dataIndex: ["nav", "system_symbol"],
      key: "current_system",
    },
    {
      title: "Current Waypoint",
      dataIndex: ["nav", "waypoint_symbol"],
      key: "current_waypoint",
    },
    {
      title: "Flight Mode",
      dataIndex: ["nav", "flight_mode"],
      key: "flight_mode",
    },
    {
      title: "Navigation Status",
      dataIndex: ["nav", "status"],
      key: "nav_status",
    },
    {
      title: "Cargo Capacity",
      dataIndex: ["cargo", "capacity"],
      key: "cargo_capacity",
    },
    {
      title: "Cargo Units",
      dataIndex: ["cargo", "units"],
      key: "cargo_units",
    },
    {
      title: "Fuel Current",
      dataIndex: ["fuel", "current"],
      key: "fuel_current",
    },
    {
      title: "Fuel Capacity",
      dataIndex: ["fuel", "capacity"],
      key: "fuel_capacity",
    },
    {
      title: "Cooldown Expiration",
      dataIndex: "cooldown_expiration",
      key: "cooldown_expiration",
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="All Ships" />
      <h2>All Ships</h2>
      <Table dataSource={Object.values(ships)} columns={columns} />;
    </div>
  );
}

export default Ships;
