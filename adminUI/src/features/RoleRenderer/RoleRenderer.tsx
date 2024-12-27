import { Link } from "react-router-dom";
import { SystemShipRole } from "../../models/ship";

const RoleRenderer = ({ role }: { role: SystemShipRole }) => {
  const renderContract = () => {
    if (role.type === "Contract" && role.data !== null) {
      const firstPart = role.data[0].slice(0, 3);
      const lastPart = role.data[0].slice(-3);
      return (
        <span>
          <Link to={`/contracts/${role.data[0]}`}>
            <span>{firstPart}</span>
            <span>...</span>
            <span>{lastPart}</span>
          </Link>
          <span> ({role.data[1]})</span>
        </span>
      );
    }
    return null;
  };

  const renderTrader = () => {
    if (role.type === "Trader" && role.data !== null) {
      return (
        <span>
          <span>{role.data[0]}</span>
          <span> ({role.data[1]})</span>
        </span>
      );
    }
    return null;
  };

  const renderMining = () => {
    if (role.type === "Mining" && role.data !== null) {
      return <span>{role.data}</span>;
    }
    return null;
  };

  return (
    <div>
      <span>{role.type}</span>{" "}
      {renderContract() || renderTrader() || renderMining()}
    </div>
  );
};

export default RoleRenderer;
