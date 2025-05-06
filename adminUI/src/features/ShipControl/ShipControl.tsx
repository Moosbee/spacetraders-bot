import {
  AutoComplete,
  Button,
  Descriptions,
  Input,
  InputNumber,
  Select,
  Space,
  Spin,
  Switch,
  Tooltip,
} from "antd";
import { useState } from "react";
import { useDispatch } from "react-redux";
import { backendUrl } from "../../data";
import { TradeSymbol } from "../../models/api";
import RustShip, { ShipInfo, SystemShipRoles } from "../../models/ship";
import { useAppSelector } from "../../redux/hooks";
import { setShip } from "../../redux/slices/shipSlice";
import { selectSystem } from "../../redux/slices/systemSlice";
import { message } from "../../utils/antdMessage";

function ShipControl({ ship }: { ship: RustShip }) {
  const system = useAppSelector((state) =>
    selectSystem(state, ship.nav.system_symbol)
  );
  const waypoints = system?.waypoints || [];

  const dispatch = useDispatch();

  const [navWaypointSymbol, setNavWaypointSymbol] = useState<string>(
    ship.nav.waypoint_symbol
  );

  const [jumpWaypointSymbol, setJumpWaypointSymbol] = useState<string>(
    ship.nav.waypoint_symbol
  );

  const [role, setRole] = useState<string>(ship.role);
  const [active, setActive] = useState<boolean>(ship.active);
  const [tradeSymbol, setTradeSymbol] = useState<TradeSymbol | undefined>(
    undefined
  );
  const [tradeAmount, setTradeAmount] = useState<number>(0);
  return (
    <Descriptions
      title={`Control ${ship.symbol}`}
      size="small"
      column={2}
      bordered
      items={[
        {
          label: "Role",
          key: "role",
          span: 2,
          children: (
            <Space>
              <Select
                value={role}
                onChange={(v) => {
                  fetch(`http://${backendUrl}/ship/${ship.symbol}/role`, {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ role: v }),
                  })
                    .then((response) => response.json())
                    .then((data: ShipInfo) => {
                      console.log("denden", data);
                      setRole(data.role);
                      message.success(
                        `Role changed to ${data.role} for ${ship.symbol}`
                      );
                    });
                }}
                style={{ minWidth: "8rem" }}
                options={Object.values(SystemShipRoles)
                  .filter((role) => role !== "TempTrader")
                  .map((role) => ({
                    label: role,
                    value: role,
                  }))}
              />
              <Spin spinning={role !== ship.role} />
            </Space>
          ),
        },
        {
          label: "Active",
          key: "active",
          children: (
            <Space>
              <Switch
                checked={active}
                onChange={() => {
                  fetch(
                    `http://${backendUrl}/ship/${ship.symbol}/toggleActivation`,
                    {
                      method: "POST",
                    }
                  )
                    .then((response) => response.json())
                    .then((data: ShipInfo) => {
                      console.log("denden", data);
                      setActive(data.active);

                      message.success(`Activation toggled for ${ship.symbol}`);
                    });
                }}
              />
              <Spin spinning={active !== ship.active} />
            </Space>
          ),
        },
        {
          label: "Orbit",
          key: "toggleOrbit",
          children: (
            <Button
              disabled={!(ship.role == "Manuel")}
              onClick={() => {
                fetch(`http://${backendUrl}/ship/${ship.symbol}/toggleOrbit`, {
                  method: "POST",
                })
                  .then((response) => response.json())
                  .then((data) => {
                    console.log("denden", data);
                    dispatch(setShip(data));

                    message.success(`Orbit toggled for ${ship.symbol}`);
                  });
              }}
            >
              {ship.nav.status !== "DOCKED" ? "Dock" : "Orbit"}
            </Button>
          ),
        },
        {
          label: "Navigate",
          key: "navigate",
          span: 2,
          children: (
            <Space>
              <AutoComplete
                disabled={!(ship.role == "Manuel")}
                value={navWaypointSymbol}
                onChange={setNavWaypointSymbol}
                style={{ minWidth: "8rem" }}
                options={(waypoints || []).map((w) => ({
                  label: <Tooltip title={w.waypoint_type}>{w.symbol}</Tooltip>,
                  value: w.symbol,
                }))}
                showSearch
              />
              <Button
                disabled={
                  !(
                    ship.role == "Manuel" &&
                    navWaypointSymbol !== "" &&
                    navWaypointSymbol !== ship.nav.waypoint_symbol
                  )
                }
                onClick={() => {
                  fetch(`http://${backendUrl}/ship/${ship.symbol}/navigate`, {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ waypointSymbol: navWaypointSymbol }),
                  })
                    .then((response) => response.json())
                    .then((data) => {
                      console.log("denden", data);
                      message.success(
                        `Started navigation to ${navWaypointSymbol}`
                      );
                    });
                }}
              >
                Navigate
              </Button>
            </Space>
          ),
        },
        {
          label: "Jump",
          key: "Jump",
          span: 2,
          children: (
            <Space>
              <Input
                disabled={!(ship.role == "Manuel")}
                value={jumpWaypointSymbol}
                onChange={(e) => setJumpWaypointSymbol(e.target.value)}
                style={{ minWidth: "8rem" }}
              />
              <Button
                disabled={
                  !(
                    ship.role == "Manuel" &&
                    jumpWaypointSymbol !== "" &&
                    jumpWaypointSymbol !== ship.nav.waypoint_symbol
                  )
                }
                onClick={() => {
                  fetch(`http://${backendUrl}/ship/${ship.symbol}/jump`, {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({
                      waypointSymbol: jumpWaypointSymbol,
                    }),
                  })
                    .then((response) => response.json())
                    .then((data) => {
                      console.log("denden", data);
                      message.success(
                        `Started jumped to ${jumpWaypointSymbol}`
                      );
                    });
                }}
              >
                Navigate
              </Button>
            </Space>
          ),
        },
        {
          label: "Purchase",
          key: "purchase",
          span: 2,
          children: (
            <Space>
              <Select
                disabled={!(ship.role == "Manuel")}
                style={{ minWidth: "8rem" }}
                options={Object.values(TradeSymbol).map((w) => ({
                  label: w,
                  value: w,
                }))}
                showSearch
                value={tradeSymbol}
                onChange={setTradeSymbol}
              />
              <InputNumber
                disabled={!(ship.role == "Manuel")}
                min={0}
                max={ship.cargo.capacity - ship.cargo.units}
                value={tradeAmount}
                onChange={(v) => setTradeAmount(v || 0)}
                changeOnWheel
                style={{ width: "4rem" }}
              />
              <Button
                disabled={!(ship.role == "Manuel")}
                onClick={() => {
                  fetch(
                    `http://${backendUrl}/ship/${ship.symbol}/purchaseCargo`,
                    {
                      method: "POST",
                      headers: { "Content-Type": "application/json" },
                      body: JSON.stringify({ tradeSymbol, units: tradeAmount }),
                    }
                  )
                    .then((response) => response.json())
                    .then((data) => {
                      console.log("denden", data);
                      message.success(
                        `Purchased ${tradeAmount} ${tradeSymbol}`
                      );
                    });
                }}
              >
                Purchase
              </Button>
            </Space>
          ),
        },
        {
          label: "Chart",
          key: "chart",
          span: 2,
          children: (
            <Button
              disabled={!(ship.role == "Manuel")}
              onClick={() => {
                fetch(`http://${backendUrl}/ship/${ship.symbol}/chart`, {
                  method: "POST",
                  headers: { "Content-Type": "application/json" },
                })
                  .then((response) => response.json())
                  .then((data) => {
                    console.log("denden", data);
                    message.success(`Charted ${ship.symbol}`);
                  });
              }}
            >
              Create Chart
            </Button>
          ),
        },
      ]}
    />
  );
}

export default ShipControl;
