import { useQuery } from "@apollo/client/react";
import { Button, Space } from "antd";
import { useMemo } from "react";
import { useParams } from "react-router-dom";
import MarketTradeHistory, {
  TradeGoodEntry,
} from "../features/MarketTradeHistory/MarketTradeHistory";
import PageTitle from "../features/PageTitle";
import { GET_WAYPOINT } from "../graphql/queries";

function WaypointMarketHistory() {
  const { systemID } = useParams();
  const { waypointID } = useParams();

  const { loading, data, refetch } = useQuery(GET_WAYPOINT, {
    variables: { waypointSymbol: waypointID || "" },
  });

  const waypoint = data?.waypoint;

  // Flatten marketTradeGoods history items into a single array with symbol+type
  const tradeHistory = useMemo(() => {
    if (!waypoint?.marketTradeGoods?.items) return [];
    const result: TradeGoodEntry[] = [];
    for (const good of waypoint.marketTradeGoods.items) {
      if (!good.history?.items) continue;
      for (const item of good.history.items) {
        result.push({
          symbol: good.symbol,
          type: good.type,
          createdAt: item.createdAt,
          purchasePrice: item.purchasePrice,
          sellPrice: item.sellPrice,
          tradeVolume: item.tradeVolume,
          supply: item.supply,
          activity: item.activity,
        });
      }
    }
    return result;
  }, [waypoint]);

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title={`Waypoint ${waypointID}`} />
      <Space>
        <h2>
          Waypoint {waypointID} in {systemID}
        </h2>
        <Button onClick={() => refetch()} loading={loading}>
          Reload
        </Button>
      </Space>
      {tradeHistory.length > 0 && <MarketTradeHistory history={tradeHistory} />}
    </div>
  );
}

export default WaypointMarketHistory;
