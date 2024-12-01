import { useParams } from "react-router-dom";

function Ship() {
  const { shipID } = useParams();
  return (
    <div>
      <h2>Ship {shipID}</h2>
    </div>
  );
}

export default Ship;
