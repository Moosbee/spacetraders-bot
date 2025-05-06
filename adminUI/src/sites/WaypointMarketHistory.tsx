import { Button, Space } from "antd";
import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { backendUrl } from "../data";
import MarketTradeHistory from "../features/MarketTradeHistory/MarketTradeHistory";
import PageTitle from "../features/PageTitle";
import { WaypointResponse } from "../models/SQLWaypoint";

function WaypointMarketHistory() {
  const { systemID } = useParams();
  const { waypointID } = useParams();

  const [waypoint, setWaypoint] = useState<WaypointResponse | null>(null);

  useEffect(() => {
    fetch(`http://${backendUrl}/waypoints/${waypointID}`)
      .then((response) => response.json())
      .then((data) => {
        console.log("waypoint", data);

        setWaypoint(data);
      });
  }, [waypointID]);

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title={`Waypoint ${waypointID}`} />
      <Space>
        <h2>
          Waypoint {waypointID} in {systemID}
        </h2>
        <Button
          onClick={() => {
            fetch(`http://${backendUrl}/waypoints/${waypointID}`)
              .then((response) => response.json())
              .then((data) => {
                console.log("waypoint", data);

                setWaypoint(data);
              });
          }}
        >
          Reload
        </Button>
      </Space>
      {waypoint?.trade_good_history &&
        waypoint.trade_good_history.length > 0 && (
          <>
            <MarketTradeHistory history={waypoint.trade_good_history} />
          </>
        )}
    </div>
  );
}

export default WaypointMarketHistory;
