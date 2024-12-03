import type { MenuProps } from "antd";
import { Avatar, Badge, Button, Dropdown, Flex, Space, theme } from "antd";
import type { AntHeaderHeader } from "../MyApp";
import useMyStore from "../store";
import FaIcon from "./FontAwsome/FaIcon";

function MyHeader({ Header }: { Header: typeof AntHeaderHeader }) {
  const isDarkMode = useMyStore((state) => state.darkMode);
  const myAgent = useMyStore((state) => state.myAgent);

  const setDarkMode = useMyStore((state) => state.setDarkMode);

  const shipSymbol = useMyStore((state) => state.selectedShipSymbol);
  const waypointSymbol = useMyStore((state) => state.selectedWaypointSymbol);
  const systemSymbol = useMyStore((state) => state.selectedSystemSymbol);

  const websocketConnected = useMyStore((state) => state.websocketConnected);

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
        <Space>
          <Avatar>{myAgent.symbol.slice(0, 1)}</Avatar>
          {myAgent.symbol}
          <span>{myAgent.credits.toLocaleString()}$</span>
          <Badge
            status={websocketConnected ? "success" : "error"}
            text={`Websocket ${
              websocketConnected ? "connected" : "not connected"
            }`}
          />
        </Space>
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
        <Dropdown trigger={["click"]} menu={{ items: settingsItems }}>
          <Button>
            <FaIcon type="solid" icon="fa-gear" /> Settings
          </Button>
        </Dropdown>
      </Flex>
    </Header>
  );
}

export default MyHeader;
