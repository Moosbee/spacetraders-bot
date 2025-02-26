import { App, ConfigProvider, Layout, theme } from "antd";
import { Route, Routes } from "react-router-dom";
import "./MyApp.css";
import MyHeader from "./features/myHeader";
import MySider from "./features/mySider";
import BulkActions from "./sites/BulkActions";
import Contract from "./sites/Contract";
import Contracts from "./sites/Contracts";
import ErrorPage from "./sites/ErrorPage";
import MarketTransactions from "./sites/MarketTransactions";
import Ship from "./sites/Ship";
import Ships from "./sites/Ships";
import System from "./sites/System";
import Systems from "./sites/Systems";
import TradeRoutes from "./sites/TradeRoutes";
import Waypoint from "./sites/Waypoint";
import WpMap from "./sites/WaypointMap";
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

                    <Route path="/system/:systemID" element={<System />} />
                    <Route
                      path="/system/:systemID/:waypointID"
                      element={<Waypoint />}
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
                      path="/transactions/market"
                      element={<MarketTransactions />}
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
