import PageTitle from "../features/PageTitle";
import SystemsMap from "../features/SystemsMap/SystemsMap";

function SysMap() {
  return (
    <div style={{ width: "100%", height: "100%", position: "relative" }}>
      <PageTitle title={`Systems Map`} />
      {/* <MapHolder zoomMax={10000}> */}
      <SystemsMap zoomMax={10000} zoomMin={1} />
      {/* </MapHolder> */}
    </div>
  );
}

export default SysMap;
