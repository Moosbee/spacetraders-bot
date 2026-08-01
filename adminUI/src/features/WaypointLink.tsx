import type { LinkProps } from "react-router-dom";
import { Link } from "react-router-dom";
import type { Prettify } from "../utils/utils";

function WaypointLink({
  systemSymbol,
  waypoint,
  children,
  ...props
}: Prettify<
  Omit<
    {
      waypoint: string;
      systemSymbol?: string;
    } & LinkProps,
    "to"
  >
>) {
  if (!systemSymbol) {
    systemSymbol = waypoint.split("-", 2).join("-");
  }

  return (
    <Link to={`/system/${systemSymbol}/${waypoint}`} {...props}>
      {children}
    </Link>
  );
}

function WaypointLinkWithSystem({
  systemSymbol,
  waypoint,
  ...props
}: Prettify<
  Omit<
    {
      waypoint: string;
      systemSymbol?: string;
    } & LinkProps,
    "to" | "children"
  >
>) {
  if (!systemSymbol) {
    systemSymbol = waypoint.split("-", 2).join("-");
  }
  return (
    <span>
      <Link to={`/system/${systemSymbol}`} {...props}>
        {systemSymbol}
      </Link>
      <Link to={`/system/${systemSymbol}/${waypoint}`} {...props}>
        {waypoint.replace(systemSymbol || "", "")}
      </Link>
    </span>
  );
}

export default WaypointLink;
export { WaypointLinkWithSystem };
