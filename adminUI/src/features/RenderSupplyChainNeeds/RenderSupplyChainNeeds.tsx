import { DeepPartial } from "@apollo/client/utilities";
import { Link } from "react-router-dom";
import { TradeSymbol, TradeSymbolInfo } from "../../gql/graphql";
import { chartColorTradeSymbol } from "../../utils/chartColors";

const blacklist = new Set([
  TradeSymbol.AluminumOre,
  TradeSymbol.IronOre,
  TradeSymbol.CopperOre,
  TradeSymbol.SiliconCrystals,
  TradeSymbol.QuartzSand,
  TradeSymbol.IceWater,
  TradeSymbol.GoldOre,
  TradeSymbol.AmmoniaIce,
  TradeSymbol.PreciousStones,
  TradeSymbol.PlatinumOre,
  TradeSymbol.SilverOre,
  TradeSymbol.Diamonds,
  TradeSymbol.Hydrocarbon,
  TradeSymbol.LiquidHydrogen,
  TradeSymbol.LiquidNitrogen,
]);

function RenderSupplyChainNeeds(
  needs: DeepPartial<TradeSymbolInfo> | undefined,
  ignoreBlacklist = false,
) {
  return (
    <ul className="list-none m-0 p-0 flex flex-col w-fit" key={needs?.symbol}>
      {(needs?.requires?.items ?? [])
        .filter((t) => t?.symbol)
        .map((need) => (
          <li
            key={need?.symbol}
            className="list-none p-[0.3em] m-[0.2em] m-l-[0.5em] border border-(--tradeSymbolColor) flex flex-row items-center justify-between"
            style={
              {
                "--depth": need?.requiredBy?.items?.length || 0,
                "--tradeSymbolColor": chartColorTradeSymbol(
                  need?.symbol ?? TradeSymbol.PreciousStones,
                ),
              } as React.CSSProperties
            }
          >
            <Link
              to={`/supplyChain/${need?.symbol}`}
              className="not-hover:text-(--tradeSymbolColor)!"
            >
              {need?.symbol}
            </Link>

            {need?.symbol && (!blacklist.has(need.symbol) || ignoreBlacklist)
              ? RenderSupplyChainNeeds(need)
              : null}
          </li>
        ))}
    </ul>
  );
}

export default RenderSupplyChainNeeds;
