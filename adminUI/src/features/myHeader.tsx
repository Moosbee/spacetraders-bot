import { useQuery } from "@apollo/client/react";
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
  Spin,
  theme,
  Tooltip,
} from "antd";
import { Link } from "react-router-dom";
import { GET_API_COUNT, GET_MY_AGENT_MINI_INFO } from "../graphql/queries";
import { type AntHeaderHeader } from "../MyApp";
import { useAppDispatch, useAppSelector } from "../redux/hooks";
import {
  selectConnectWebsocket,
  selectDarkMode,
  selectWebsocketConnected,
  setConnectWebsocket,
  setDarkMode,
} from "../redux/slices/configSlice";
import {
  selectSelectedShipSymbol,
  selectSelectedSystemSymbol,
  selectSelectedWaypointSymbol,
} from "../redux/slices/mapSlice";
import FaIcon from "./FontAwsome/FaIcon";
import MoneyDisplay from "./MonyDisplay";
import { WaypointLinkWithSystem } from "./WaypointLink";

function MyHeader({ Header }: { Header: typeof AntHeaderHeader }) {
  const isDarkMode = useAppSelector(selectDarkMode);

  const shipSymbol = useAppSelector(selectSelectedShipSymbol);
  const waypointSymbol = useAppSelector(selectSelectedWaypointSymbol);
  const systemSymbol = useAppSelector(selectSelectedSystemSymbol);

  const websocketConnected = useAppSelector(selectWebsocketConnected);

  const connectWebsocket = useAppSelector(selectConnectWebsocket);

  const dispatch = useAppDispatch();

  const {
    loading: apiCountLoading,
    data: apiCount,
    refetch: apiCountRefetch,
  } = useQuery(GET_API_COUNT, {
    initialFetchPolicy: "standby",
  });

  const {
    loading: myAgentLoading,
    data: myAgentData,
    refetch: myAgentRefetch,
  } = useQuery(GET_MY_AGENT_MINI_INFO, {
    initialFetchPolicy: "standby",
  });

  const {
    token: { colorBgContainer, colorTextDescription },
  } = theme.useToken();

  const settingsItems: MenuProps["items"] = [
    {
      key: "darkMode",
      onClick: (e) => {
        e.domEvent.stopPropagation();
        e.domEvent.preventDefault();
        dispatch(setDarkMode(!isDarkMode));
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
    {
      key: "websocket",
      onClick: (e) => {
        e.domEvent.preventDefault();
        dispatch(setConnectWebsocket(!connectWebsocket));
      },
      label: (
        <Space>
          {connectWebsocket ? "Disconnect Websocket" : "Connect Websocket"}
          <Badge status={connectWebsocket ? "success" : "error"} />
        </Space>
      ),
      icon: <FaIcon type="solid" icon="fa-right-from-bracket" />,
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
                {myAgentData?.runInfo.agent?.accountId}
              </Col>
              <Col span={12} style={{ textAlign: "center" }}>
                {myAgentData?.runInfo.agent?.symbol}
              </Col>
              <Col span={12} style={{ textAlign: "center" }}>
                {myAgentData?.runInfo.agent?.shipCount} Ships
              </Col>
              <Col span={12} style={{ textAlign: "center" }}>
                {myAgentData?.runInfo.agent?.startingFaction}
              </Col>
              <Col span={12} style={{ textAlign: "center" }}>
                <WaypointLinkWithSystem
                  waypoint={myAgentData?.runInfo.agent?.headquarters || ""}
                  className="text-blue-400! hover:text-blue-200!"
                />
              </Col>
              <Col span={12} style={{ textAlign: "center" }}>
                <Space>
                  <Badge status={websocketConnected ? "success" : "error"} />
                  {websocketConnected ? "Online" : "Offline"}
                </Space>
              </Col>
              <Col span={12} style={{ textAlign: "center" }}>
                <MoneyDisplay
                  amount={myAgentData?.runInfo.agent?.credits || -1}
                />
              </Col>
              <Col span={12} style={{ textAlign: "center" }}>
                <Button
                  onClick={() => {
                    myAgentRefetch();
                  }}
                >
                  Refresh
                </Button>
              </Col>
              <Col span={12} style={{ textAlign: "center" }}>
                <MoneyDisplay
                  amount={myAgentData?.budget.reservedAmount || -1}
                />
              </Col>
            </Row>
          }
        >
          <Link
            to={`/agents/${myAgentData?.runInfo.agent?.symbol}`}
            style={{ color: "inherit" }}
          >
            <Space>
              <Spin size="small" spinning={myAgentLoading}>
                <Avatar>
                  {myAgentData?.runInfo.agent?.symbol.slice(0, 1)}
                </Avatar>
              </Spin>
              {myAgentData?.runInfo.agent?.symbol}
              <Badge status={websocketConnected ? "success" : "error"} />
              <MoneyDisplay
                amount={myAgentData?.runInfo.agent?.credits || -1}
              />
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
              apiCountRefetch();
            }}
            loading={apiCountLoading}
          >
            API Count: {apiCount?.apiCounts || 0}
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
