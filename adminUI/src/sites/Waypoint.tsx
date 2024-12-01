import { useParams } from "react-router-dom";

function Waypoint() {
  const { systemID } = useParams();
  const { waypointID } = useParams();

  return (
    <div>
      <h2>
        Waypoint {waypointID} in system {systemID}
      </h2>
    </div>
  );
}

export default Waypoint;
