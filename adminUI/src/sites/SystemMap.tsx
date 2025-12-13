import { useQuery } from "@apollo/client/react";
import PageTitle from "../features/PageTitle";
import SystemsMap from "../features/SystemsMap/SystemsMap";
import { GET_SYSTEM_MAP_DATA } from "../graphql/queries";

function SysMap() {
  const { error, data, dataState } = useQuery(GET_SYSTEM_MAP_DATA);

  if (dataState != "complete") return <p>Loading...</p>;
  if (error) return <p>Error: {error.message}</p>;

  return (
    <div style={{ width: "100%", height: "100%", position: "relative" }}>
      <PageTitle title={`Systems Map`} />
      {/* <MapHolder zoomMax={10000}> */}
      <SystemsMap zoomMax={10000} zoomMin={0.5} data={data} />
      {/* </MapHolder> */}
    </div>
  );
}

export default SysMap;
