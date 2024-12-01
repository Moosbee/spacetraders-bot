import { Button } from "antd";
import { Link, useParams } from "react-router-dom";
import useMyStore, { backendUrl } from "../store";

function System() {
  const { systemID } = useParams();
  const Waypoints = useMyStore((state) => state.waypoints[systemID || ""]);
  const setWaypoints = useMyStore((state) => state.setWaypoints);
  return (
    <div>
      <h2>System {systemID}</h2>
      <Link to={`/map/system/${systemID}`}>Map</Link>
      <Button
        onClick={() => {
          fetch(`http://${backendUrl}/waypoints`)
            .then((response) => response.json())
            .then(setWaypoints);
        }}
      >
        Reload
      </Button>
      {Waypoints && (
        <ul>
          {Object.keys(Waypoints).map((waypointID) => (
            <li key={waypointID}>
              <Link to={`/waypoint/${systemID}/${waypointID}`}>
                {waypointID}
              </Link>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export default System;
