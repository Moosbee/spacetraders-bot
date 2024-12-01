import { useEffect, useRef, useState, type ReactElement } from "react";
import { SystemWaypoint, Waypoint, WaypointType } from "../../models/api";
import useMyStore from "../../store";
import FaIcon from "../FontAwsome/FaIcon";
import NounIcon from "../FontAwsome/NounIcon";
import classes from "./WaypointMapWaypoint.module.css";

//TODO change color to antd color and dark/light mode

const waypointIcons: Record<
  WaypointType,
  { icon: ReactElement; color: string }
> = {
  PLANET: {
    icon: <FaIcon type="solid" icon="fa-earth-oceania" />,
    color: "brown",
  },
  GAS_GIANT: {
    icon: <FaIcon type="solid" icon="fa-planet-ringed" />,
    color: "lightblue",
  },
  MOON: {
    icon: <FaIcon type="solid" icon="fa-moon" />,
    color: "grey",
  },
  ORBITAL_STATION: {
    icon: <NounIcon name="space-station" />,
    color: "yellow",
  },
  JUMP_GATE: {
    icon: <FaIcon type="solid" icon="fa-bullseye-pointer" />,
    color: "yellow",
  },
  ASTEROID_FIELD: {
    icon: <NounIcon name="asteroid-field" />,
    color: "lightgrey",
  },
  ASTEROID: {
    icon: <NounIcon name="asteroid" />,
    color: "lightgrey",
  },
  ENGINEERED_ASTEROID: {
    icon: <NounIcon name="asteroid_2" />,
    color: "lightgrey",
  },
  ASTEROID_BASE: {
    icon: <FaIcon type="solid" icon="fa-planet-ringed" />,
    color: "yellow",
  },
  NEBULA: {
    icon: <NounIcon name="nebula" />,
    color: "currentColor",
  },
  DEBRIS_FIELD: {
    icon: <FaIcon type="solid" icon="fa-sparkles" />,
    color: "red",
  },
  GRAVITY_WELL: {
    icon: <FaIcon type="solid" icon="fa-arrows-minimize" />,
    color: "green",
  },
  ARTIFICIAL_GRAVITY_WELL: {
    icon: <FaIcon type="solid" icon="fa-arrows-to-circle" />,
    color: "yellowgreen",
  },
  FUEL_STATION: {
    icon: <FaIcon type="solid" icon="fa-gas-pump" />,
    color: "yellow",
  },
};

function WaypointMapWaypoint({
  systemSymbol,
  waypoint,
  xOne,
  yOne,
}: {
  systemSymbol: string;
  waypoint: Waypoint | SystemWaypoint;
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

  const color = waypointIcons[waypoint.type].color;
  const waypointIcon = waypointIcons[waypoint.type].icon;

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
