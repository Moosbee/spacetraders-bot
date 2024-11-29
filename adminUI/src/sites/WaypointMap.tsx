import { useParams } from "react-router-dom";
import MapHolder from "../features/MapHolder/MapHolder";
import PageTitle from "../features/PageTitle";
import WaypointMap from "../features/WaypointMap/WaypointMap";

function WpMap() {
  const { systemID } = useParams();

  return (
    <div style={{ width: "100%", height: "100%", position: "relative" }}>
      <PageTitle title={`${systemID} Map`} />
      {systemID && (
        <MapHolder>
          <WaypointMap systemID={systemID} />
        </MapHolder>
      )}
    </div>
  );
}

export default WpMap;
