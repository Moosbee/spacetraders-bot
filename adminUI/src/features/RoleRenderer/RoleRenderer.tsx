import { Link } from "react-router-dom";
import { GetAllShipsQuery } from "../../gql/graphql";
import Timer from "../Timer/Timer";

const RoleRenderer = ({
  status,
}: {
  status: GetAllShipsQuery["ships"][number]["status"];
}) => {
  const inner = status.status;
  const typename = inner.__typename;

  const renderContract = () => {
    if (typename === "ContractStatus") {
      const firstPart = inner.contractId?.slice(0, 3);
      const lastPart = inner.contractId?.slice(-3);
      return (
        <span>
          <Link to={`/contracts/${inner.contractId}`}>
            <span>{firstPart}</span>
            <span>...</span>
            <span>{lastPart}</span>
          </Link>
          {inner.waitingForManager ? "*" : ""} ({inner.cycle})
          <br />
          <span>
            Shipment {inner.runId} -{" "}
            {inner.shippingStatus === "DELIVERING" && "Delivering"}
            {inner.shippingStatus === "IN_TRANSIT_TO_DELIVERY" &&
              "Transit to D"}
            {inner.shippingStatus === "IN_TRANSIT_TO_PURCHASE" &&
              "Transit to P"}
            {inner.shippingStatus === "PURCHASING" && "Purchasing"}
          </span>
        </span>
      );
    }
    return null;
  };

  const renderTrader = () => {
    if (typename === "TraderStatus") {
      return (
        <span>
          <span>
            {inner.shipmentId}
            {inner.waitingForManager ? "*" : ""} ({inner.cycle})
          </span>
          <br />
          <span>{inner.shippingStatus}</span>
        </span>
      );
    }
    return null;
  };

  const renderMining = () => {
    if (typename === "MiningStatus") {
      const assignment = inner.assignment;
      const assignTypename = assignment.__typename;
      switch (assignTypename) {
        case "ExtractorAssignment":
        case "SiphonerAssignment":
          return (
            <span>
              {assignTypename.replace("Assignment", "")} -{" "}
              {assignment.extractions}
              <br />
              {assignment.state} - {assignment.waypointSymbol}
            </span>
          );

        case "TransporterAssignment":
          return (
            <span>
              {assignTypename.replace("Assignment", "")} - {assignment.cycles}
              <br />
              {assignment.waypointSymbol}
            </span>
          );

        case "IdleAssignment":
        case "SurveyorAssignment":
        case "UselessAssignment":
        default:
          return <span>{assignTypename.replace("Assignment", "")}</span>;
      }
    }
    return null;
  };

  const renderConstruction = () => {
    if (typename === "ConstructionStatus") {
      return (
        <span>
          {inner.shipmentId}
          {inner.waitingForManager ? "*" : ""} ({inner.cycle})
          <br />
          {inner.shippingStatus}
        </span>
      );
    }
    return null;
  };

  const renderScraper = () => {
    if (typename === "ScraperStatus") {
      return (
        <span>
          {inner.waitingForManager ? "*" : ""} ({inner.cycle})
          <br />
          {inner.waypointSymbol}{" "}
          {inner.scrapDate && <Timer time={inner.scrapDate} />}
        </span>
      );
    }
    return null;
  };

  const renderTransfer = () => {
    if (typename === "TransferStatus") {
      return (
        <span>
          ({inner.assignmentId})
          <br />
          {inner.fleetId}{" "}
          <Link to={`/system/${inner.systemSymbol}`}>{inner.systemSymbol}</Link>
        </span>
      );
    }
    return null;
  };

  const renderCharting = () => {
    if (typename === "ChartingStatus") {
      return (
        <span>
          {inner.waitingForManager ? "*" : ""} ({inner.cycle})
          <br />
          {inner.waypointSymbol}
        </span>
      );
    }
    return null;
  };

  return (
    <div>
      <span>{typename?.replace("Status", "")}</span>{" "}
      {renderContract() ||
        renderTrader() ||
        renderMining() ||
        renderConstruction() ||
        renderScraper() ||
        renderTransfer() ||
        renderCharting()}
    </div>
  );
};

export default RoleRenderer;
