import { ReactFlowProvider } from "@xyflow/react";
import { GetSystemMarketsQuery } from "../../gql/graphql";
import MarketSupplyChainVisual from "./MarketSupplyChainVisual";

function MarketSupplyChainFlow({
  marketTrades,
}: {
  marketTrades: GetSystemMarketsQuery["system"]["marketTrades"]["items"];
}) {
  return (
    <ReactFlowProvider>
      <MarketSupplyChainVisual marketTrades={marketTrades} />
    </ReactFlowProvider>
  );
}

export default MarketSupplyChainFlow;
