import { App, ConfigProvider, Layout, theme } from "antd";
import { Route, Routes } from "react-router-dom";
import "./App.css";
import MyHeader from "./features/myHeader";
import MySider from "./features/mySider";
import ErrorPage from "./sites/ErrorPage";
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
                    <Route
                      path="/"
                      element={<Main></Main>}
                      errorElement={<ErrorPage />}
                    />
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
