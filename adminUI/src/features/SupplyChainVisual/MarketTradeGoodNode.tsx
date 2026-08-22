import { Handle, NodeProps, Position } from "@xyflow/react";
import { theme } from "antd";
import { GetSystemMarketsQuery, SupplyLevel } from "../../gql/graphql";
import { cn } from "../../utils/utils";
import MoneyDisplay from "../MonyDisplay";

function MarketTradeGoodNode({
  data: { tradeGood },
}: NodeProps & {
  data: {
    tradeGood?: GetSystemMarketsQuery["system"]["marketTrades"]["items"][number];
    label?: string;
  };
  type: "marketTradeGood";
}) {
  const {
    token: { colorBgElevated },
  } = theme.useToken();
  if (!tradeGood) return null;
  return (
    <div
      style={{ background: colorBgElevated }}
      className={cn(
        "rounded-md p-1  border-transparent in-[.selected]:border-4 in-[.selected]:border-yellow-500",
        tradeGood.marketTradeGood?.supply === SupplyLevel.Scarce &&
          "border-b-4 border-b-red-500",
        tradeGood.marketTradeGood?.supply === SupplyLevel.Limited &&
          "border-b-4 border-b-orange-500",
        tradeGood.marketTradeGood?.supply === SupplyLevel.Moderate &&
          "border-b-4 border-b-amber-500",
        tradeGood.marketTradeGood?.supply === SupplyLevel.High &&
          "border-b-4 border-b-blue-500",
        tradeGood.marketTradeGood?.supply === SupplyLevel.Abundant &&
          "border-b-4 border-b-green-500",
      )}
    >
      <div className="flex flex-col">
        <span>
          {tradeGood.waypointSymbol} {tradeGood.type} {tradeGood.symbol}
        </span>
        <span className="flex justify-between">
          <MoneyDisplay amount={tradeGood.marketTradeGood?.sellPrice || 0} />
          {tradeGood.marketTradeGood?.supply}{" "}
          {tradeGood.marketTradeGood?.tradeVolume}
          <MoneyDisplay
            amount={tradeGood.marketTradeGood?.purchasePrice || 0}
          />
        </span>
      </div>
      <Handle type="source" position={Position.Right} />
      <Handle type="target" position={Position.Left} />
    </div>
  );
}

export default MarketTradeGoodNode;
