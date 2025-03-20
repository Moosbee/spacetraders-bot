import { useEffect, useRef, useState } from "react";
import { SQLSystem } from "../../models/SQLSystem";
import useMyStore from "../../store";
import { systemIcons } from "../../utils/waypointColors";
import classes from "./WaypointMapSystem.module.css";

function WaypointMapSystem({
  system,
  xOne,
  yOne,
}: {
  system: SQLSystem;
  xOne: number;
  yOne: number;
}) {
  const [size, setSize] = useState(16);
  const textboxRef = useRef<HTMLDivElement>(null);
  const selectedSystem = useMyStore((state) => state.selectedSystemSymbol);

  const setSelectedSystemSymbol = useMyStore(
    (state) => state.setSelectedSystemSymbol
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

  const color = systemIcons[system.system_type].color;
  const waypointIcon = systemIcons[system.system_type].icon;

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
      className={`${classes.waypointContainer} ${classes.star}`}
      onClick={() => {
        if (selectedSystem === system.symbol) {
          setSelectedSystemSymbol(undefined);
          return;
        }
        setSelectedSystemSymbol(system.symbol);
      }}
      onDoubleClick={() => {
        window.open(
          `/system/${system.symbol}`,
          "_blank"
          //  "popup:true"
        );
      }}
    >
      <div className={classes.waypointIcon} ref={textboxRef}>
        {waypointIcon}
      </div>
      <div className={classes.waypointInfo}>
        {/* {waypoint.x}, {waypoint.y} */}
        {/* {waypoint?.symbol.replace(system.symbol + "-", "")} */}
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

export default WaypointMapSystem;
