import { Button, Card, Col, Flex, theme } from "antd";
import { toPng } from "html-to-image";
import { useMemo, useRef, useState } from "react";
import { Link } from "react-router-dom";
import {
  GetSystemsWithFleetsOrShipsQuery,
  TradeSymbol,
} from "../../gql/graphql";
import { cn } from "../../utils/utils";
import { systemIcons } from "../../utils/waypointColors";
import WaypointMap from "../WaypointMap/WaypointMap";

function SystemCard({
  system,
  mapScale = "1/2",
  canvasSize = 2000,
}: {
  system: GetSystemsWithFleetsOrShipsQuery["systems"]["items"][number];
  mapScale: "2/1" | "1/1" | "1/2" | "1/3" | "1/4" | "1/8" | "1/16";
  canvasSize?: number;
}) {
  const { icon, color } =
    systemIcons[system.systemType] ?? systemIcons.BLACK_HOLE;

  const [loading, setLoading] = useState(false);

  const marketTrades = useMemo(() => {
    const opportunities = (system?.marketTrades.items || []).reduce(
      (a, b) => {
        a[b.symbol] = a[b.symbol] || [0, 0, 0, 0];
        a[b.symbol][0] += 1;
        switch (b.type) {
          case "IMPORT":
            a[b.symbol][1] += 1;
            break;
          case "EXPORT":
            a[b.symbol][2] += 1;
            break;
          case "EXCHANGE":
            a[b.symbol][3] += 1;
            break;
        }
        return a;
      },
      {} as Record<TradeSymbol, [number, number, number, number]>,
    );

    return opportunities;
  }, [system?.marketTrades]);

  const [
    totalMarketOpportunities,
    marketOpportunities,
    strictMarketOpportunities,
  ] = useMemo(() => {
    const total = Object.entries(marketTrades)
      .filter((s) => s[0] !== "FUEL")
      .reduce((a, b) => a + (b[1][0] * (b[1][0] - 1)) / 2, 0);
    const strict = Object.entries(marketTrades)
      .filter((s) => s[0] !== "FUEL")
      .reduce((a, b) => a + b[1][1] * b[1][2], 0); // only trades between export and import
    const market = Object.entries(marketTrades)
      .filter((s) => s[0] !== "FUEL")
      .reduce(
        (a, b) => a + b[1][1] * b[1][2] + b[1][1] * b[1][3] + b[1][3] * b[1][2],
        0,
      ); // trades between import and import additionally with exchange trades
    return [total, market, strict];
  }, [marketTrades]);

  const maxRange = useMemo(() => {
    return system.minimumSpanningTree
      .toSorted((a, b) => a.distance - b.distance)
      .toReversed()[0]?.distance;
  }, [system.minimumSpanningTree]);

  const {
    token: { colorBgBase },
  } = theme.useToken();

  const mapRef = useRef<HTMLDivElement>(null);

  const handleCapture = async () => {
    if (!mapRef.current) return;
    setLoading(true);
    try {
      console.log("Capturing screenshot of map:", system.symbol);
      // const canvas = await html2canvas(mapRef.current, { allowTaint: true });
      // const dataURL = canvas.toDataURL("image/png");

      const dataURL = await toPng(mapRef.current, {
        canvasHeight: canvasSize,
        canvasWidth: canvasSize,
      });

      const link = document.createElement("a");
      link.href = dataURL;
      link.download = `screenshot-${system.symbol}.png`;
      link.click();
    } catch (error) {
      console.error("Error capturing screenshot:", error);
    }
    setLoading(false);
  };

  return (
    <Col key={system.symbol} xs={24} sm={12} lg={8} xl={6}>
      <Card
        variant="outlined"
        title={
          <Flex align="center" justify="space-between" gap={8}>
            <Flex align="center" gap={8}>
              <span
                style={{ color }}
                className="flex h-6 w-6 items-center justify-center text-xl"
              >
                {icon}
              </span>
              <Link to={`/system/${system.symbol}`}>{system.symbol}</Link>
            </Flex>
            <Button onClick={handleCapture} loading={loading}>
              Capture
            </Button>
          </Flex>
        }
      >
        <div className="overflow-hidden w-full aspect-square">
          <div
            className={cn("origin-top-left aspect-square", {
              "scale-200 w-[50%]": mapScale === "2/1",
              "scale-100 w-full": mapScale === "1/1",
              "scale-50 w-[200%]": mapScale === "1/2",
              "scale-33 w-[300%]": mapScale === "1/3",
              "scale-25 w-[400%]": mapScale === "1/4",
              "scale-12 w-[800%]": mapScale === "1/8",
              "scale-6 w-[1600%]": mapScale === "1/16",
            })}
          >
            <div
              style={{ backgroundColor: colorBgBase }}
              className="p-4 rounded-2xl overflow-hidden w-full aspect-square"
              ref={mapRef}
            >
              <div className="w-full h-full relative">
                <WaypointMap
                  systemData={system}
                  systemWaypoints={system.waypoints.items}
                  systemShips={[]}
                  config={{
                    showShips: false,
                    highlightSelectedShip: false,
                    highlightSelectedWaypoint: false,
                    showAutoPilot: "NONE",
                    showOnlyMarkets: false,
                  }}
                />
              </div>
            </div>
          </div>
        </div>
        <div className="mt-4">
          Markets {system.systemAnalyzation.totalMarketCount}
          <br />
          Inner System {system.systemAnalyzation.innerSystemCount}
          <br />
          Market Opportunities {totalMarketOpportunities} (Strict:{" "}
          {strictMarketOpportunities}) (Market: {marketOpportunities})
          <br />
          Max Range {maxRange.toFixed(2) ?? 0}
        </div>
      </Card>
    </Col>
  );
}

export default SystemCard;
