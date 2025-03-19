import { useEffect, useMemo, useState } from "react";
import { SQLSystem } from "../../models/SQLSystem";
import { backendUrl } from "../../store";
import SystemMapSystem from "../SystemMapSystem/SystemMapSystem";
import classes from "./SystemsMap.module.css";

function SystemsMap() {
  const [systems, setSystems] = useState<SQLSystem[]>([]);

  useEffect(() => {
    fetch(`http://${backendUrl}/systems`)
      .then((response) => response.json())
      .then((data) => {
        console.log("systems", data);

        setSystems(data);
      });
  }, []);

  const calcSystems: { system: SQLSystem; xOne: number; yOne: number }[] =
    useMemo(() => {
      const [wpMinX, wpMinY, wpMaxX, wpMaxY] =
        calculateSystemBoundaries(systems);

      return systems.map((s) => ({
        system: s,
        xOne: ((s.x - wpMinX) / (wpMaxX - wpMinX)) * 100,
        yOne: ((s.y - wpMinY) / (wpMaxY - wpMinY)) * 100,
      }));
    }, [systems]);

  return (
    <div className={classes.systemMapIn}>
      {/* <WaypointMapSystem system={System.system} xOne={50} yOne={50} /> */}
      {calcSystems.map((s) => (
        <SystemMapSystem
          key={s.system.symbol + "system"}
          system={s.system}
          xOne={s.xOne}
          yOne={s.yOne}
        />
      ))}
    </div>
  );
}

function calculateSystemBoundaries(systemssArr: SQLSystem[]) {
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
