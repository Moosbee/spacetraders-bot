import type { MenuProps } from "antd";
import {
  Avatar,
  Badge,
  Button,
  Col,
  Dropdown,
  Flex,
  Row,
  Space,
  theme,
  Tooltip,
} from "antd";
import { useState } from "react";
import { Link } from "react-router-dom";
import { DbAgent } from "../models/Agent";
import type { AntHeaderHeader } from "../MyApp";
import useMyStore, { backendUrl } from "../store";
import FaIcon from "./FontAwsome/FaIcon";
import MoneyDisplay from "./MonyDisplay";

function MyHeader({ Header }: { Header: typeof AntHeaderHeader }) {
  const isDarkMode = useMyStore((state) => state.darkMode);
  const myAgent = useMyStore((state) => state.myAgent);

  const setDarkMode = useMyStore((state) => state.setDarkMode);
  const setAgent = useMyStore((state) => state.setAgent);

  const shipSymbol = useMyStore((state) => state.selectedShipSymbol);
  const waypointSymbol = useMyStore((state) => state.selectedWaypointSymbol);
  const systemSymbol = useMyStore((state) => state.selectedSystemSymbol);

  const websocketConnected = useMyStore((state) => state.websocketConnected);

  const [apiCount, setApiCount] = useState(0);

  const {
    token: { colorBgContainer, colorTextDescription },
  } = theme.useToken();

  const settingsItems: MenuProps["items"] = [
    {
      key: "darkMode",
      onClick: (e) => {
        e.domEvent.stopPropagation();
        e.domEvent.preventDefault();
        setDarkMode(!isDarkMode);
      },
      label: `${isDarkMode ? "Light" : "Dark"}-Mode`,
      icon: <FaIcon type="solid" icon={isDarkMode ? "fa-moon" : "fa-sun"} />,
    },
    {
      key: "popUp",

      onClick: (e) => {
        e.domEvent.preventDefault();
        window.open(window.location.pathname, undefined, "popup:true");
      },
      label: "Pop Up",
      icon: <FaIcon type="solid" icon="fa-window-restore" />,
    },
  ];

  return (
    <Header
      style={{
        position: "sticky",
        top: 0,
        zIndex: 1,
        width: "100%",
        background: colorBgContainer,
        padding: "0 24px",
      }}
    >
      <Flex gap="middle" align="center" justify="space-between">
        <Tooltip
          title={
            <Row gutter={[2, 2]}>
              <Col span={24} style={{ textAlign: "center" }}>
                {myAgent.account_id}
              </Col>
              <Col span={12} style={{ textAlign: "center" }}>
                {myAgent.symbol}
              </Col>
              <Col span={12} style={{ textAlign: "center" }}>
                {myAgent.ship_count} Ships
              </Col>
              <Col span={12} style={{ textAlign: "center" }}>
                {myAgent.starting_faction}
              </Col>
              <Col span={12} style={{ textAlign: "center" }}>
                {myAgent.headquarters}
              </Col>
              <Col span={12} style={{ textAlign: "center" }}>
                <Space>
                  <Badge status={websocketConnected ? "success" : "error"} />
                  {websocketConnected ? "Online" : "Offline"}
                </Space>
              </Col>
              <Col span={12} style={{ textAlign: "center" }}>
                <MoneyDisplay amount={myAgent.credits} />
              </Col>
              <Col span={12} style={{ textAlign: "center" }}>
                <Button
                  onClick={() => {
                    fetch(`http://${backendUrl}/agents`)
                      .then((res) => res.json())
                      .then((res) => {
                        for (const agent of res as DbAgent[]) {
                          if (agent.account_id) {
                            setAgent(agent);
                            break;
                          }
                        }
                      });
                  }}
                >
                  Refresh
                </Button>
              </Col>
            </Row>
          }
        >
          <Link to={`/agents/${myAgent.symbol}`} style={{ color: "inherit" }}>
            <Space>
              <Avatar>{myAgent.symbol.slice(0, 1)}</Avatar>
              {myAgent.symbol}
              <Badge status={websocketConnected ? "success" : "error"} />
              <MoneyDisplay amount={myAgent.credits} />
            </Space>
          </Link>
        </Tooltip>
        <div>
          {systemSymbol && (
            <span>
              <FaIcon
                type="solid"
                icon="fa-solar-system"
                style={{
                  color: colorTextDescription,
                }}
              />{" "}
              <b>{systemSymbol}</b>
            </span>
          )}
          {systemSymbol && waypointSymbol && `   `}
          {waypointSymbol && (
            <span>
              <FaIcon
                type="solid"
                icon="fa-planet-moon"
                style={{
                  color: colorTextDescription,
                }}
              />{" "}
              <b>
                {waypointSymbol.waypointSymbol
                  .replace(systemSymbol || "", "")
                  .replace("-", "")}
              </b>
            </span>
          )}
          {shipSymbol && waypointSymbol && `   `}
          {shipSymbol && (
            <span>
              <FaIcon
                type="solid"
                icon="fa-rocket-launch"
                style={{
                  color: colorTextDescription,
                }}
              />{" "}
              <b>{shipSymbol}</b>
            </span>
          )}
        </div>
        <Space>
          <Button
            onClick={() => {
              fetch(`http://${backendUrl}/insights/apiCounter`)
                .then((response) => response.json())
                .then((data) => setApiCount(data.counter));
            }}
          >
            API Count: {apiCount}
          </Button>
          <Dropdown trigger={["click"]} menu={{ items: settingsItems }}>
            <Button>
              <FaIcon type="solid" icon="fa-gear" /> Settings
            </Button>
          </Dropdown>
        </Space>
      </Flex>
    </Header>
  );
}

export default MyHeader;
