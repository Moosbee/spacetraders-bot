import { App, ConfigProvider, Layout, theme } from "antd";
import { Route, Routes } from "react-router-dom";
import "./MyApp.css";
import MyHeader from "./features/myHeader";
import MySider from "./features/mySider";
import Agent from "./sites/Agent";
import Agents from "./sites/Agents";
import BulkActions from "./sites/BulkActions";
import ConstructionMaterials from "./sites/ConstructionMaterials";
import ConstructionShipments from "./sites/ConstructionShipments";
import Contract from "./sites/Contract";
import Contracts from "./sites/Contracts";
import ErrorPage from "./sites/ErrorPage";
import MarketTransactions from "./sites/MarketTransactions";
import MiningAssignments from "./sites/MiningAssignments";
import PossibleTrades from "./sites/PossibleTrades";
import Ship from "./sites/Ship";
import Ships from "./sites/Ships";
import ShipsToPurchase from "./sites/ShipsToPurchase";
import System from "./sites/System";
import SysMap from "./sites/SystemMap";
import Systems from "./sites/Systems";
import TradeRoutes from "./sites/TradeRoutes";
import Waypoint from "./sites/Waypoint";
import WpMap from "./sites/WaypointMap";
import WaypointMarketHistory from "./sites/WaypointMarketHistory";
import Main from "./sites/main";
import useMyStore from "./store";
import MessageAntD from "./utils/message";
import WorkerLoader from "./workers/WorkerLoader";
const { Header, Content, Sider } = Layout;

export { Header as AntHeaderHeader, Sider as AntSiderSider };

function MyApp() {
  const {
    token: { borderRadiusLG },
  } = theme.useToken();

  const { defaultAlgorithm, darkAlgorithm } = theme;
  const isDarkMode = useMyStore((state) => state.darkMode);

  return (
    <>
      <MessageAntD />
      <WorkerLoader />

      <ConfigProvider
        theme={{
          algorithm: isDarkMode ? darkAlgorithm : defaultAlgorithm,
        }}
      >
        <App>
          <Layout>
            <MyHeader Header={Header} />
            <Layout>
              <MySider Slider={Sider}></MySider>
              <Layout>
                <Content
                  style={{
                    padding: 0,
                    // padding: 24,
                    margin: 0,
                    minHeight: "calc(100vh - 64px)",
                    borderRadius: borderRadiusLG,
                  }}
                >
                  <Routes>
                    <Route path="/" element={<Main></Main>} />
                    <Route path="/ships" element={<Ships />} />
                    <Route path="/systems" element={<Systems />} />
                    <Route path="/systems/map" element={<SysMap></SysMap>} />

                    <Route path="/system/:systemID" element={<System />} />
                    <Route
                      path="/system/:systemID/:waypointID"
                      element={<Waypoint />}
                    />
                    <Route
                      path="/system/:systemID/:waypointID/marketHistory"
                      element={<WaypointMarketHistory />}
                    />
                    <Route
                      path="/map/system/:systemID"
                      element={<WpMap></WpMap>}
                    />
                    <Route path="/ships/:shipID" element={<Ship />} />
                    <Route path="/bulk" element={<BulkActions />} />
                    <Route path="/contracts" element={<Contracts />} />
                    <Route
                      path="/contracts/:contractID"
                      element={<Contract />}
                    />
                    <Route path="/tradeRoutes" element={<TradeRoutes />} />
                    <Route
                      path="/possibleTrades"
                      element={<PossibleTrades />}
                    />
                    <Route
                      path="/transactions/market"
                      element={<MarketTransactions />}
                    />
                    <Route path="/agents/:agentID" element={<Agent />} />
                    <Route path="/agents" element={<Agents />} />

                    <Route
                      path="/construction/Materials"
                      element={<ConstructionMaterials />}
                    />
                    <Route
                      path="/construction/shipments"
                      element={<ConstructionShipments />}
                    />

                    <Route
                      path="/shipsToPurchase"
                      element={<ShipsToPurchase />}
                    />

                    <Route
                      path="/miningAssignments"
                      element={<MiningAssignments />}
                    />

                    <Route path="*" element={<ErrorPage />} />
                  </Routes>
                </Content>
              </Layout>
            </Layout>
          </Layout>
        </App>
      </ConfigProvider>
    </>
  );
}

export default MyApp;
