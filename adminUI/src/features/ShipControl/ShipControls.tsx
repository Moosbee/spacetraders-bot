import { Empty } from "antd";
import { ShipData } from "../../sites/Ship";

function ShipControls({ ship }: { ship: ShipData }) {
  void ship;
  return (
    <Empty description="Live ship data is not loaded yet. Manual controls will appear here once available." />
  );
}

export default ShipControls;
