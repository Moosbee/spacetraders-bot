import { useEffect, useRef, useState } from "react";
import { SQLWaypoint } from "../../models/SQLWaypoint";
import useMyStore from "../../store";
import { waypointIcons } from "../../utils/waypointColors";
import classes from "./WaypointMapWaypoint.module.css";

//TODO change color to antd color and dark/light mode

function WaypointMapWaypoint({
  systemSymbol,
  waypoint,
  xOne,
  yOne,
}: {
  systemSymbol: string;
  waypoint: SQLWaypoint;
  xOne: number;
  yOne: number;
}) {
  const [size, setSize] = useState(16);
  const textboxRef = useRef<HTMLDivElement>(null);
  const selectedWaypoint = useMyStore((state) => state.selectedWaypointSymbol);

  const setSelectedWaypointSymbol = useMyStore(
    (state) => state.setSelectedWaypointSymbol
  );

  function outputsize() {
    if (!textboxRef.current) return;

    setSize(textboxRef.current.offsetWidth);
  }

  useEffect(() => {
    if (!textboxRef.current) return;
    const observe = new ResizeObserver(outputsize);
    observe.observe(textboxRef.current);

    return () => {
      observe.disconnect();
    };
  }, []);

  const color = waypointIcons[waypoint.waypoint_type].color;
  const waypointIcon = waypointIcons[waypoint.waypoint_type].icon;

  return (
    <div
      style={
        {
          left: xOne + "%",
          top: yOne + "%",
          "--waypoint-icon-size": `${Math.floor(size * 0.85)}px`,
          "--waypoint-icon-color": color,
        } as React.CSSProperties
      }
      className={`${classes.waypointContainer} ${
        waypoint ? classes.waypoint : classes.star
      } ${
        selectedWaypoint?.waypointSymbol === waypoint?.symbol && waypoint
          ? classes.active
          : ""
      }`}
      onClick={() => {
        if (selectedWaypoint?.waypointSymbol === waypoint.symbol) {
          setSelectedWaypointSymbol(undefined);
          return;
        }

        setSelectedWaypointSymbol({
          waypointSymbol: waypoint.symbol,
          systemSymbol: waypoint.symbol,
        });
      }}
      onDoubleClick={() => {
        window.open(
          `/system/${systemSymbol}/${waypoint.symbol}`,
          "_blank"
          // "popup:true",
        );
      }}
    >
      <div className={classes.waypointIcon} ref={textboxRef}>
        {waypointIcon}
      </div>
      <div className={classes.waypointInfo}>
        {/* {waypoint.x}, {waypoint.y} */}
        {waypoint?.symbol.replace(systemSymbol + "-", "")}
        {/* <br />
        <div
          className={classes.waypointInfoMore}
          style={
            {
              "--waypoint-icon-size": `${Math.floor(size * 0.85)}px`,
            } as React.CSSProperties
          }
        >
          {waypoint?.type}
        </div> */}
      </div>
    </div>
  );
}

export default WaypointMapWaypoint;
