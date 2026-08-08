import { useQuery } from "@apollo/client/react";
import { Button, Space } from "antd";
import { useMemo } from "react";
import { Link, useParams } from "react-router-dom";
import MarketTradeHistory, {
  TradeGoodEntry,
} from "../features/MarketTradeHistory/MarketTradeHistory";
import PageTitle from "../features/PageTitle";
import WaypointLink from "../features/WaypointLink";
import { GET_WAYPOINT_HISTORY } from "../graphql/queries";

function WaypointMarketHistory() {
  const { systemID } = useParams();
  const { waypointID } = useParams();

  const { loading, data, refetch } = useQuery(GET_WAYPOINT_HISTORY, {
    variables: { waypointSymbol: waypointID || "" },
  });

  const waypoint = data?.waypoint;

  // Flatten marketTradeGoods history items into a single array with symbol+type
  const tradeHistory = useMemo(() => {
    if (!waypoint?.marketTrades?.items) return [];
    const result: TradeGoodEntry[] = [];
    for (const good of waypoint.marketTrades.items) {
      if (!good?.marketTradeGood?.history?.items) continue;
      for (const item of good.marketTradeGood.history.items) {
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
          Waypoint{" "}
          <WaypointLink waypoint={waypointID ?? ""} systemSymbol={systemID}>
            {waypointID}
          </WaypointLink>{" "}
          in <Link to={`/system/${systemID}`}>{systemID}</Link>
        </h2>
        <Button onClick={() => refetch()} loading={loading}>
          Reload
        </Button>
      </Space>
      {tradeHistory.length > 0 && (
        <MarketTradeHistory
          history={tradeHistory}
          marketTrades={waypoint?.marketTrades?.items || []}
        />
      )}
    </div>
  );
}

export default WaypointMarketHistory;
