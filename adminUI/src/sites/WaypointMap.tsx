import { useQuery } from "@apollo/client/react";
import { Button, Dropdown, Result, Spin } from "antd";
import { useParams } from "react-router-dom";
import MapHolder from "../features/MapHolder/MapHolder";
import PageTitle from "../features/PageTitle";
import WaypointMap from "../features/WaypointMap/WaypointMap";
import { GET_SYSTEM_MAP } from "../graphql/queries";

function WpMap() {
  const { systemID } = useParams();
  const { loading, error, data, dataState, refetch } = useQuery(
    GET_SYSTEM_MAP,
    {
      variables: { systemSymbol: systemID || "" },
    },
  );

  const items = [
    {
      key: "1",
      label: <Button onClick={() => refetch()}>Refetch</Button>,
    },
  ];

  return (
    <div style={{ width: "100%", height: "100%", position: "relative" }}>
      <PageTitle title={`${systemID} Map`} />
      {/* <div > */}
      <Spin spinning={loading} fullscreen />
      {dataState == "complete" && !error && (
        <Dropdown menu={{ items }} trigger={["contextMenu"]}>
          <div>
            <MapHolder>
              <WaypointMap
                systemData={data.system}
                systemShips={data.system.ships}
              />
            </MapHolder>
          </div>
        </Dropdown>
      )}
      {error && (
        <Result
          status="error"
          title="Failed to load system map"
          subTitle="Please check your network connection or try again later."
          extra={[
            <Button key="refetch" onClick={() => refetch()} type="primary">
              Try again
            </Button>,
          ]}
        ></Result>
      )}
    </div>
  );
}

export default WpMap;
