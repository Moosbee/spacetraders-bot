import { theme } from "antd";
import { useEffect, useRef, useState } from "react";
import RustShip from "../../models/ship";
import useMyStore from "../../store";
import FaIcon from "../FontAwsome/FaIcon";
import classes from "./WaypointMapShip.module.css";

function WaypointMapShip({
  ship,
  xOne,
  yOne,
}: {
  ship: RustShip;
  xOne: number;
  yOne: number;
}) {
  const [size, setSize] = useState(16);
  const textboxRef = useRef<HTMLDivElement>(null);
  const selectedship = useMyStore((state) => state.selectedShipSymbol);
  const setSelectedship = useMyStore((state) => state.setSelectedShipSymbol);

  useEffect(() => {
    if (!textboxRef.current) return;
    const observe = new ResizeObserver(outputsize);
    observe.observe(textboxRef.current);

    return () => {
      observe.disconnect();
    };
  }, []);

  function outputsize() {
    if (!textboxRef.current) return;

    setSize(textboxRef.current.offsetWidth);
  }

  const {
    token: { colorText },
  } = theme.useToken();

  const color = colorText;
  const shipIcon =
    ship.nav.status === "IN_ORBIT" ? (
      <FaIcon type="solid" icon="fa-rocket" />
    ) : ship.nav.status === "DOCKED" ? (
      <FaIcon type="solid" icon="fa-rocket" />
    ) : (
      <FaIcon type="solid" icon="fa-rocket-launch" />
    );

  return (
    <div
      style={
        {
          left: xOne + "%",
          top: yOne + "%",
          "--ship-icon-size": `${Math.floor(size * 0.85)}px`,
          "--ship-icon-color": color,
        } as React.CSSProperties
      }
      className={`${classes.shipContainer} ${
        ship ? classes.ship : classes.star
      } ${selectedship === ship?.symbol && ship ? classes.active : ""}`}
      onClick={() => {
        if (ship) {
          if (selectedship === ship.symbol) {
            setSelectedship(undefined);
            return;
          }
          setSelectedship(ship.symbol);
        }
      }}
      onDoubleClick={() => {
        if (ship) {
          window.open(`/ships/${ship.symbol}`, "_blank", "popup:true");
        }
      }}
    >
      <div className={classes.shipIcon} ref={textboxRef}>
        {shipIcon}
      </div>
      <div className={classes.shipInfo}>
        {/* {ship.x}, {ship.y} */}
        {ship?.symbol.split("-")[0][0] + "-" + ship?.symbol.split("-")[1]}
        {/* <br />
        <div
          className={classes.shipInfoMore}
          style={
            {
              "--ship-icon-size": `${Math.floor(size * 0.85)}px`,
            } as React.CSSProperties
          }
        >
          {ship?.type}
        </div> */}
      </div>
    </div>
  );
}

export default WaypointMapShip;
