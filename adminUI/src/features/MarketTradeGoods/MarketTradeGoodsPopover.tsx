import {
  DownloadOutlined,
  TruckOutlined,
  UploadOutlined,
} from "@ant-design/icons";
import { Flex } from "antd";
import { Link } from "react-router-dom";
import { GetSystemQuery } from "../../gql/graphql";
import MoneyDisplay from "../MonyDisplay";

function MarketTradeGoodsPopover({
  marketTrades,
}: {
  marketTrades: GetSystemQuery["system"]["waypoints"]["items"][number]["marketTrades"]["items"];
}) {
  return (
    <Flex gap={1} vertical>
      {marketTrades.filter((t) => t.type === "EXCHANGE").length > 0 && (
        <span className="font-bold">EXCHANGE</span>
      )}
      {marketTrades
        .filter((t) => t.type === "EXCHANGE")
        .map((trade_good) => (
          <Flex justify="space-between" key={trade_good.symbol}>
            <Link to={`/supplyChain/${trade_good.symbol}`}>
              {trade_good.symbol}
            </Link>
            <Flex gap={1} justify="end">
              <span className="text-nowrap">
                <UploadOutlined />{" "}
                <MoneyDisplay
                  amount={trade_good.marketTradeGood?.purchasePrice || 0}
                />
              </span>
              |
              <span className="text-nowrap">
                <DownloadOutlined />{" "}
                <MoneyDisplay
                  amount={trade_good.marketTradeGood?.sellPrice || 0}
                />
              </span>
              |
              <span className="text-nowrap">
                <TruckOutlined /> {trade_good.marketTradeGood?.tradeVolume}
              </span>
              |<span>{trade_good.marketTradeGood?.supply.slice(0, 3)}</span>
            </Flex>
          </Flex>
        ))}
      {marketTrades.filter((t) => t.type === "IMPORT").length > 0 && (
        <span className="font-bold">IMPORT</span>
      )}

      {marketTrades
        .filter((t) => t.type === "IMPORT")
        .map((trade_good) => (
          <Flex justify="space-between" key={trade_good.symbol}>
            <Link to={`/supplyChain/${trade_good.symbol}`}>
              {trade_good.symbol}
            </Link>
            <Flex gap={1} justify="end">
              <span className="text-nowrap">
                <UploadOutlined />{" "}
                <MoneyDisplay
                  amount={trade_good.marketTradeGood?.purchasePrice || 0}
                />
              </span>
              |
              <span className="font-bold text-nowrap">
                <DownloadOutlined />{" "}
                <MoneyDisplay
                  amount={trade_good.marketTradeGood?.sellPrice || 0}
                />
              </span>
              |
              <span className="text-nowrap">
                <TruckOutlined /> {trade_good.marketTradeGood?.tradeVolume}
              </span>
              |<span>{trade_good.marketTradeGood?.supply.slice(0, 3)}</span>
            </Flex>
          </Flex>
        ))}
      {marketTrades.filter((t) => t.type === "EXPORT").length > 0 && (
        <span className="font-bold">EXPORT</span>
      )}

      {marketTrades
        .filter((t) => t.type === "EXPORT")
        .map((trade_good) => (
          <Flex justify="space-between" key={trade_good.symbol}>
            <Link to={`/supplyChain/${trade_good.symbol}`}>
              {trade_good.symbol}
            </Link>
            <Flex gap={1} justify="end">
              <span className="text-nowrap font-bold">
                <UploadOutlined />{" "}
                <MoneyDisplay
                  amount={trade_good.marketTradeGood?.purchasePrice || 0}
                />
              </span>
              |
              <span className="text-nowrap">
                <DownloadOutlined />{" "}
                <MoneyDisplay
                  amount={trade_good.marketTradeGood?.sellPrice || 0}
                />
              </span>
              |
              <span className="text-nowrap">
                <TruckOutlined /> {trade_good.marketTradeGood?.tradeVolume}
              </span>
              |<span>{trade_good.marketTradeGood?.supply.slice(0, 3)}</span>
            </Flex>
          </Flex>
        ))}
      {marketTrades.filter((t) => t.type === "EXPORT").length > 0 && (
        <span className="font-bold">MAPPING</span>
      )}
      <div className="flex flex-col">
        {marketTrades
          .filter((t) => t.type === "EXPORT")
          .map((trade_good) => (
            <div
              key={trade_good.symbol + "EXPORT" + trade_good.type}
              className={`flex justify-between border-t-2 border-t-current`}
            >
              <div className="flex flex-col">
                {trade_good.tradeSymbolInfo.requires.items.map((t) => (
                  <div
                    key={t.symbol}
                    className={`${
                      marketTrades.some(
                        (e) => e.type === "IMPORT" && e.symbol == t.symbol,
                      )
                        ? "text-current"
                        : "text-red-700"
                    }`}
                  >
                    <Link to={`/supplyChain/${t.symbol}`}>{t.symbol}</Link>
                  </div>
                ))}
              </div>
              <div className="flex items-center">
                <Link to={`/supplyChain/${trade_good.symbol}`}>
                  {trade_good.symbol}
                </Link>
              </div>
            </div>
          ))}
      </div>
    </Flex>
  );
}

export default MarketTradeGoodsPopover;
