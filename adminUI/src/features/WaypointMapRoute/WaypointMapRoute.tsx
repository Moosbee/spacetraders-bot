import { ShipNavFlightMode } from "../../models/api";
import { useAppSelector } from "../../redux/hooks";
import { selectDarkMode } from "../../redux/slices/configSlice";
import classes from "./WaypointMapRoute.module.css";

function WaypointMapRoute({
  line,
  size,
  mode,
}: {
  line: {
    x1: number;
    y1: number;
    x2: number;
    y2: number;
  };
  size: number;
  mode: ShipNavFlightMode | "MSP";
}) {
  const darkMode = useAppSelector(selectDarkMode);

  return (
    <line
      style={
        {
          "--stroke-width": `${Math.min(0.2, 200 / size)}px`,
          color:
            mode === "MSP"
              ? darkMode
                ? "oklch(55.4% 0.046 257.417)"
                : "oklch(44.6% 0.043 257.281)"
              : mode === "BURN"
                ? darkMode
                  ? "yellow"
                  : "#7D00FF"
                : mode === "DRIFT"
                  ? "red"
                  : "green",
        } as React.CSSProperties
      }
      x1={line.x1}
      y1={line.y1}
      x2={line.x2}
      y2={line.y2}
      className={classes.route}
    ></line>
  );
}

export default WaypointMapRoute;
