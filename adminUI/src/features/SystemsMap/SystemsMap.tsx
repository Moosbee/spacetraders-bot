import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { GetSystemMapDataQuery } from "../../gql/graphql";
import { useAppDispatch, useAppSelector } from "../../redux/hooks";
import {
  selectSelectedSystemSymbol,
  setSelectedSystemSymbol,
} from "../../redux/slices/mapSlice";
import { systemMapDrawStyle } from "../../utils/systemMapColors";
import { systemIcons } from "../../utils/waypointColors";
import classes from "./SystemsMap.module.css";

type GQLSystem = Omit<
  GetSystemMapDataQuery["systems"]["items"][number],
  "waypoints" | "fleets"
> & {
  waypoints: GetSystemMapDataQuery["systems"]["items"][number]["waypoints"]["items"];
  fleets: GetSystemMapDataQuery["systems"]["items"][number]["fleets"]["items"];
};

type GQLJumpConnection =
  GetSystemMapDataQuery["jumpConnections"]["items"][number];

function resizeCanvas(canvas: HTMLCanvasElement) {
  const { width, height } = canvas.getBoundingClientRect();

  if (canvas.width !== width || canvas.height !== height) {
    const { devicePixelRatio: ratio = 1 } = window;
    const context = canvas.getContext("2d");
    if (!context) {
      return false;
    }
    canvas.width = width * ratio;
    canvas.height = height * ratio;
    context.scale(ratio, ratio);
    return true;
  }

  return false;
}

function drawSystems(
  canvas: HTMLCanvasElement,
  systems: Record<string, { system: GQLSystem; xOne: number; yOne: number }>,
  jumpGates: {
    underConstructionA: boolean;
    underConstructionB: boolean;
    pointASymbol: string;
    pointBSymbol: string;
    fromA: boolean;
    fromB: boolean;
  }[],
  zoom: number,
  top: number,
  left: number,
  selectedSystem: string | undefined,
  config: {
    minFleetsHighlighted?: number;
    minShipsHighlighted?: number;
    minWaypointsHighlighted?: number;
    minShipyardsHighlighted?: number;
    minMarketplacesHighlighted?: number;
    highlightSelectedSystem?: boolean;
    highlightStarterSystems?: boolean;
  },
  style: {
    selectedSystemHighlightColor: string;
    workingJumpGateLineColor: string;
    blockedJumpGateLineColor: string;
    shipHighlightColor: string;
    fleetHighlightColor: string;
    marketplaceHighlightColor: string;
    shipyardHighlightColor: string;
    waypointHighlightColor: string;
    starterSystemHighlightColor: string;
  },
) {
  const context = canvas.getContext("2d");
  if (!context) {
    return;
  }

  const width = canvas.clientWidth;
  const height = canvas.clientHeight;

  context.clearRect(0, 0, width, height);

  // const minRatio = Math.min(width, height);
  const maxRatio = Math.max(width, height);

  for (const {
    underConstructionA,
    underConstructionB,
    pointASymbol,
    pointBSymbol,
  } of jumpGates) {
    const systemSymbolA = pointASymbol.split("-", 2).join("-");
    const systemSymbolB = pointBSymbol.split("-", 2).join("-");
    const system_a = systems[systemSymbolA];
    const system_b = systems[systemSymbolB];
    if (!system_a || !system_b) continue;
    context.beginPath();
    context.moveTo(
      system_a?.xOne * zoom * maxRatio + left,
      system_a?.yOne * zoom * maxRatio + top,
    );
    context.lineTo(
      system_b?.xOne * zoom * maxRatio + left,
      system_b?.yOne * zoom * maxRatio + top,
    );
    if (underConstructionA || underConstructionB) {
      // context.strokeStyle = "#ff00000f";
      // context.strokeStyle = "#ff0000ff";
      context.strokeStyle = style.blockedJumpGateLineColor;
    } else {
      context.strokeStyle = style.workingJumpGateLineColor;
    }
    context.closePath();
    context.stroke();
  }

  const highLightRadiusBig = [10, 8, 7, 6, 5, 4, 3, 2];
  const highLightRadiusSmall = [5, 4, 3, 2];

  for (const { system, xOne, yOne } of Object.values(systems)) {
    const x = xOne * zoom * maxRatio + left;
    const y = yOne * zoom * maxRatio + top;
    if (x < 0 || x > width || y < 0 || y > height) continue;
    const r = Math.max(maxRatio / 3000, Math.min(Math.abs(zoom / 2) * 1, 10));

    const drawRing = (radiusMultiplier: number, color: string) => {
      context.beginPath();
      context.arc(x, y, r * radiusMultiplier, 0, 2 * Math.PI);
      context.fillStyle = color;
      context.fill();
    };

    const shipyardCount = system.waypoints.filter(
      (wp) => wp.hasShipyard,
    ).length;
    const marketplaceCount = system.waypoints.filter(
      (wp) => wp.hasMarketplace,
    ).length;
    const isStarterSystem = system.waypoints.some(
      (wp) => wp.waypointType === "ENGINEERED_ASTEROID",
    );

    const highlights = [
      {
        active: selectedSystem === system.symbol,
        selected: config.highlightSelectedSystem,
        color: style.selectedSystemHighlightColor,
      },
      {
        active: (config.minFleetsHighlighted || 0) <= system.fleets.length,
        selected: config.minFleetsHighlighted !== undefined,
        color: style.fleetHighlightColor,
      },
      {
        active: (config.minShipsHighlighted || 0) <= system.ships.length,
        selected: config.minShipsHighlighted !== undefined,
        color: style.shipHighlightColor,
      },
      {
        active:
          (config.minWaypointsHighlighted || 0) <= system.waypoints.length,
        selected: config.minWaypointsHighlighted !== undefined,
        color: style.waypointHighlightColor,
      },
      {
        active: (config.minShipyardsHighlighted || 0) <= shipyardCount,
        selected: config.minShipyardsHighlighted !== undefined,
        color: style.shipyardHighlightColor,
      },
      {
        active: (config.minMarketplacesHighlighted || 0) <= marketplaceCount,
        selected: config.minMarketplacesHighlighted !== undefined,
        color: style.marketplaceHighlightColor,
      },
      {
        active: isStarterSystem,
        selected: config.highlightStarterSystems,
        color: style.starterSystemHighlightColor,
      },
    ];

    let radiusIndex = 0;
    const activeCount = highlights.filter(
      (highlight) => highlight.active,
    ).length;
    if (activeCount) {
      for (const { active, color, selected } of highlights) {
        if (selected) {
          const currentRadius = radiusIndex++;
          if (active) {
            let multiplier = 2;
            if (activeCount < 4) {
              multiplier = highLightRadiusSmall[currentRadius] || 14;
            } else {
              multiplier = highLightRadiusBig[currentRadius] || 14;
            }
            drawRing(multiplier, color);
          }
        }
      }
    }

    context.beginPath();
    context.arc(x, y, r, 0, 2 * Math.PI);
    const color = systemIcons[system.systemType].color;
    context.fillStyle = color;
    context.fill();
    if (zoom > 10) {
      const alpha = 0.5 + (0.5 * (zoom - 20)) / 20;
      context.fillStyle = color
        .replace("rgb", "rgba")
        .replace(")", `, ${alpha})`);
      context.font = "16px serif";
      context.fillText(system.symbol, x + r * 1.3, y + r * 0.5);
    }
  }
}

function SystemsMap({
  zoomMax = 1000,
  zoomMin = 0.01,
  data,
  config = {},
}: {
  zoomMax: number;
  zoomMin: number;
  data: { systems: GQLSystem[]; jumpConnections: GQLJumpConnection[] };
  config: {
    minWaypointCount?: number;
    minFleetCount?: number;
    minShipyardCount?: number;
    minMarketplaceCount?: number;
    onlyJumpGates?: "ACCESSIBLE" | "NOT_ACCESSIBLE" | "NONE";
    minFleetsHighlighted?: number;
    minShipsHighlighted?: number;
    highlightStarterSystems?: boolean;
    minWaypointsHighlighted?: number;
    minShipyardsHighlighted?: number;
    minMarketplacesHighlighted?: number;
    highlightSelectedSystem?: boolean;
    doubleClickBehavior?: "SELECT_SYSTEM" | "GO_TO_SYSTEM" | "NONE" | undefined;
  };
}) {
  const selectedSystem = useAppSelector(selectSelectedSystemSymbol);
  const dispatch = useAppDispatch();

  const calcSystems: Record<
    string,
    { system: GQLSystem; xOne: number; yOne: number }
  > = useMemo(() => {
    const [wpMinX, wpMinY, wpMaxX, wpMaxY] = calculateSystemBoundaries(
      data.systems,
    );

    const wp: Record<
      string,
      { system: GQLSystem; xOne: number; yOne: number }
    > = {};

    for (const system of data.systems) {
      if ((system.waypoints.length || 0) < (config.minWaypointCount ?? 0))
        continue;
      if (
        system.waypoints.filter((wp) => wp.hasMarketplace).length <
        (config.minMarketplaceCount ?? 0)
      )
        continue;
      if (
        system.waypoints.filter((wp) => wp.hasShipyard).length <
        (config.minShipyardCount ?? 0)
      )
        continue;
      if (
        (config.onlyJumpGates ?? "NONE") !== "NONE" &&
        system.waypoints.filter(
          (wp) =>
            wp.waypointType === "JUMP_GATE" &&
            (config.onlyJumpGates === "ACCESSIBLE"
              ? !wp.isUnderConstruction
              : true),
        ).length === 0
      )
        continue;
      wp[system.symbol] = {
        system: system,
        xOne: (system.x - wpMinX) / (wpMaxX - wpMinX),
        yOne: (system.y - wpMinY) / (wpMaxY - wpMinY),
      };
    }

    return wp;
  }, [
    config.minMarketplaceCount,
    config.minShipyardCount,
    config.minWaypointCount,
    config.onlyJumpGates,
    data.systems,
  ]);

  const [zoom, setZoom] = useState(1);
  const [top, setTop] = useState(0);
  const [left, setLeft] = useState(0);

  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (!ref.current) return;
    const observe = new ResizeObserver(() => {
      if (!ref.current) return;

      resizeCanvas(ref.current);
      drawSystems(
        ref.current,
        calcSystems,
        data.jumpConnections,
        zoom,
        top,
        left,
        selectedSystem,
        {
          minFleetsHighlighted: config.minFleetsHighlighted,
          minShipsHighlighted: config.minShipsHighlighted,
          minWaypointsHighlighted: config.minWaypointsHighlighted,
          minShipyardsHighlighted: config.minShipyardsHighlighted,
          minMarketplacesHighlighted: config.minMarketplacesHighlighted,
          highlightSelectedSystem: config.highlightSelectedSystem,
          highlightStarterSystems: config.highlightStarterSystems,
        },
        systemMapDrawStyle,
      );
    });
    observe.observe(ref.current);

    return () => {
      observe.disconnect();
    };
  }, [
    calcSystems,
    config.highlightSelectedSystem,
    config.highlightStarterSystems,
    config.minFleetsHighlighted,
    config.minMarketplacesHighlighted,
    config.minShipsHighlighted,
    config.minShipyardsHighlighted,
    config.minWaypointsHighlighted,
    data.jumpConnections,
    left,
    selectedSystem,
    top,
    zoom,
  ]);

  useEffect(() => {
    const canvas = ref.current;
    if (!canvas) return;
    drawSystems(
      canvas,
      calcSystems,
      data.jumpConnections,
      zoom,
      top,
      left,
      selectedSystem,
      {
        minFleetsHighlighted: config.minFleetsHighlighted,
        minShipsHighlighted: config.minShipsHighlighted,
        minWaypointsHighlighted: config.minWaypointsHighlighted,
        minShipyardsHighlighted: config.minShipyardsHighlighted,
        minMarketplacesHighlighted: config.minMarketplacesHighlighted,
        highlightSelectedSystem: config.highlightSelectedSystem,
        highlightStarterSystems: config.highlightStarterSystems,
      },
      systemMapDrawStyle,
    );
  }, [
    calcSystems,
    config.highlightSelectedSystem,
    config.highlightStarterSystems,
    config.minFleetsHighlighted,
    config.minMarketplacesHighlighted,
    config.minShipsHighlighted,
    config.minShipyardsHighlighted,
    config.minWaypointsHighlighted,
    data.jumpConnections,
    left,
    selectedSystem,
    top,
    zoom,
  ]);

  const onWheel = useCallback(
    (e: WheelEvent) => {
      if (!ref.current) return;
      e.preventDefault();

      // let newZoom;
      // if (e.deltaY > 0) {
      //   newZoom = Math.max(zoom - 5, zoomMin);
      // } else {
      //   newZoom = Math.min(zoom + 5, zoomMax);
      // }

      const zoomFactor = 0.05;
      const newZoom = Math.min(
        Math.max(
          zoom + (e.deltaY > 0 ? -zoom * zoomFactor : zoom * zoomFactor),
          zoomMin,
        ),
        zoomMax,
      );
      // const zoomDiff = newZoom - zoom;

      const zoomDiff = newZoom - zoom;

      const width = ref.current.clientWidth;
      const height = ref.current.clientHeight;

      // const minRatio = Math.min(width, height);
      const maxRatio = Math.max(width, height);

      const bounding = ref.current.getBoundingClientRect();
      // this is the position of the mouse relative to the frame 0 top of the frame 1 bottom of the frame
      const mausPercentPosY = (e.clientY - bounding.y) / height;
      // this is the position of the mouse relative to the frame 0 left of the frame 1 right of the frame
      const mausPercentPosX = (e.clientX - bounding.x) / width;

      const WdH = ref.current.clientWidth / ref.current.clientHeight;
      const HdW = ref.current.clientHeight / ref.current.clientWidth;

      //height is X, width is Y

      const cursorPosY = height * mausPercentPosY;
      const cursorPosX = width * mausPercentPosX;

      const mapPosY = (cursorPosY - top) / zoom / maxRatio; // between 0 and 1
      const mapPosX = (cursorPosX - left) / zoom / maxRatio;

      // this is the ammount to move the frame up or down to compensate the change in zoom
      const topDiff =
        zoomDiff * cursorPosY * (mapPosY / mausPercentPosY) * Math.max(WdH, 1);
      // this is the ammount to move the frame left or right to compensate the change in zoom
      const leftDiff =
        zoomDiff * cursorPosX * (mapPosX / mausPercentPosX) * Math.max(HdW, 1);

      const newTop = top - topDiff;
      const newLeft = left - leftDiff;

      // const newZoomTop = zoomTop + topDiff;
      // const newZoomLeft = zoomLeft + leftDiff;

      // setZoomTop(newZoomTop);
      // setZoomLeft(newZoomLeft);

      setZoom(newZoom);
      setTop(Number.isFinite(newTop) ? newTop : 0);
      setLeft(Number.isFinite(newLeft) ? newLeft : 0);
    },
    [left, top, zoom, zoomMax, zoomMin],
  );

  useEffect(() => {
    if (ref && ref.current) {
      const rref = ref.current;
      rref.addEventListener("wheel", onWheel, false);
      return function cleanup() {
        if (ref && ref.current) {
          // eslint-disable-next-line react-hooks/exhaustive-deps
          ref.current.removeEventListener("wheel", onWheel, false);
        }
        rref.removeEventListener("wheel", onWheel, false);
      };
    }
  }, [onWheel]);

  const [lastPosX, setLastPosX] = useState(0);
  const [lastPosY, setLastPosY] = useState(0);

  const onMouseMove = (e: React.PointerEvent) => {
    if (!ref.current) return;

    if (e.buttons !== 1) return;

    const diffX = e.clientX - lastPosX;
    const diffY = e.clientY - lastPosY;

    setLastPosX(e.clientX);
    setLastPosY(e.clientY);

    // const newLeft = left + scaleNum(diffX, 0, ref.current.clientWidth, 0, 100);
    // const newTop = top + scaleNum(diffY, 0, ref.current.clientHeight, 0, 100);

    const newLeft = left + diffX;
    const newTop = top + diffY;

    setLeft(Number.isFinite(newLeft) ? newLeft : 0);
    setTop(Number.isFinite(newTop) ? newTop : 0);
  };

  return (
    <canvas
      className={classes.SystemsMap}
      ref={ref}
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === "ArrowLeft") {
          setLeft((prev) => prev + 10);
        } else if (e.key === "ArrowRight") {
          setLeft((prev) => prev - 10);
        } else if (e.key === "ArrowUp") {
          setTop((prev) => prev + 10);
        } else if (e.key === "ArrowDown") {
          setTop((prev) => prev - 10);
        } else if (e.key === "+") {
          setZoom((prev) => prev + 1);
        } else if (e.key === "-") {
          setZoom((prev) => prev - 1);
        } else if (e.key === "r") {
          setZoom(1);
          setTop(0);
          setLeft(0);
          setTop(0);
          setLeft(0);
        } else if (e.key === "g") {
          if (!selectedSystem) return;
          const toGoNav = selectedSystem;
          const system = calcSystems[toGoNav];
          if (system && ref.current) {
            const width = ref.current.width;
            const height = ref.current.height;

            // const minRatio = Math.min(width, height);
            const maxRatio = Math.max(width, height);
            const left = system.xOne * maxRatio * zoom - width / 2;
            const top = system.yOne * maxRatio * zoom - height / 2;

            // const x = system.xOne * zoom * maxRatio - width / 2;
            // const y = system.yOne * zoom * maxRatio - height / 2;
            // setLeft(-x + width / 2);
            // setTop(-y + height / 2);

            setLeft(-left);
            setTop(-top);
          }
        }
      }}
      onPointerDown={(e) => {
        setLastPosX(e.clientX);
        setLastPosY(e.clientY);
      }}
      onPointerMove={(e) => {
        onMouseMove(e);
      }}
      onDoubleClick={(e) => {
        if (!ref.current) return;
        const width = ref.current.clientWidth;
        const height = ref.current.clientHeight;

        // const minRatio = Math.min(width, height);
        const maxRatio = Math.max(width, height);

        const bounding = ref.current.getBoundingClientRect();
        // this is the position of the mouse relative to the frame 0 top of the frame 1 bottom of the frame
        const mausPosY = e.clientY - bounding.y;
        // this is the position of the mouse relative to the frame 0 left of the frame 1 right of the frame
        const mausPosX = e.clientX - bounding.x;

        const mapPosX = (mausPosX - left) / zoom / maxRatio; // between 0 and 1
        const mapPosY = (mausPosY - top) / zoom / maxRatio; // between 0 and 1

        const closestSystem = Object.values(calcSystems).reduce(
          (prev, curr) => {
            const prevDistance =
              Math.abs(curr.xOne - mapPosX) + Math.abs(curr.yOne - mapPosY);
            const currDistance =
              Math.abs(prev.xOne - mapPosX) + Math.abs(prev.yOne - mapPosY);
            return prevDistance > currDistance ? prev : curr;
          },
          Object.values(calcSystems)[0],
        );

        console.log(
          "closestSystem",
          closestSystem,
          closestSystem.system.symbol,
        );

        if (config.doubleClickBehavior === "GO_TO_SYSTEM") {
          window.open("/system/" + closestSystem.system.symbol, "_blank");
        } else if (config.doubleClickBehavior === "SELECT_SYSTEM") {
          dispatch(setSelectedSystemSymbol(closestSystem.system.symbol));
        }
      }}
    />
  );
}

function calculateSystemBoundaries(systemssArr: { x: number; y: number }[]) {
  let wpMinX = Infinity;
  let wpMinY = Infinity;
  let wpMaxX = -Infinity;
  let wpMaxY = -Infinity;
  systemssArr.forEach((w) => {
    wpMinX = Math.min(wpMinX, w.x);
    wpMinY = Math.min(wpMinY, w.y);
    wpMaxX = Math.max(wpMaxX, w.x);
    wpMaxY = Math.max(wpMaxY, w.y);
  });
  return [wpMinX, wpMinY, wpMaxX, wpMaxY];
}

export default SystemsMap;
