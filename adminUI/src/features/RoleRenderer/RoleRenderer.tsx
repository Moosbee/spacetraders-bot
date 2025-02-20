import { Link } from "react-router-dom";
import { SystemShipRole } from "../../models/ship";

const RoleRenderer = ({
  role,
  status,
}: {
  role: any;
  status: SystemShipRole;
}) => {
  const renderContract = () => {
    if (status.type === "Contract" && status.data !== null) {
      const firstPart = status.data[0].slice(0, 3);
      const lastPart = status.data[0].slice(-3);
      return (
        <span>
          <Link to={`/contracts/${status.data[0]}`}>
            <span>{firstPart}</span>
            <span>...</span>
            <span>{lastPart}</span>
          </Link>
          <span> ({status.data[1]})</span>
        </span>
      );
    }
    return null;
  };

  const renderTrader = () => {
    if (status.type === "Trader" && status.data !== null) {
      return (
        <span>
          <span>{status.data[0]}</span>
          <span> ({status.data[1]})</span>
        </span>
      );
    }
    return null;
  };

  const renderMining = () => {
    if (status.type === "Mining" && status.data !== null) {
      return <span>{status.data}</span>;
    }
    return null;
  };

  return (
    <div>
      <span>{role}</span> {renderContract() || renderTrader() || renderMining()}
    </div>
  );
};

export default RoleRenderer;
