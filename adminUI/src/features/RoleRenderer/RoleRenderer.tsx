import { Link } from "react-router-dom";
import { SystemShipRole, SystemShipRoles } from "../../models/ship";

const RoleRenderer = ({
  role,
  status,
}: {
  role: SystemShipRoles;
  status: SystemShipRole;
}) => {
  const renderContract = () => {
    if (status.type === "Contract" && status.data !== null) {
      const firstPart = status.data.contract_id?.slice(0, 3);
      const lastPart = status.data.contract_id?.slice(-3);
      return (
        <span>
          <Link to={`/contracts/${status.data.contract_id}`}>
            <span>{firstPart}</span>
            <span>...</span>
            <span>{lastPart}</span>
          </Link>{" "}
          ({status.data.cycle})
          <br />
          <span>
            Shipment {status.data.run_id} - {status.data.shipping_status}
          </span>
        </span>
      );
    }
    return null;
  };

  const renderTrader = () => {
    if (status.type === "Trader" && status.data !== null) {
      return (
        <span>
          <span>
            {status.data.shipment_id} ({status.data.cycle})
          </span>
          <br />
          <span>{status.data.shipping_status}</span>
        </span>
      );
    }
    return null;
  };

  const renderMining = () => {
    if (status.type === "Mining" && status.data !== null) {
      switch (status.data.assignment.type) {
        case "Extractor":
        case "Siphoner":
          return (
            <span>
              {status.data.assignment.type} -{" "}
              {status.data.assignment.data.extractions}
              <br />
              {status.data.assignment.data.state} -{" "}
              {status.data.assignment.data.waypoint_symbol}
            </span>
          );

        case "Transporter":
          return (
            <span>
              {status.data.assignment.type} -{" "}
              {status.data.assignment.data.cycles}
              <br />
              {status.data.assignment.data.state} -{" "}
              {status.data.assignment.data.waypoint_symbol}
            </span>
          );

        case "Idle":
        case "Surveyor":
        case "Useless":
        default:
          return <span>{status.data.assignment.type}</span>;
      }
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
