import { Agent } from "./api";
import RustShip from "./ship";

interface WsObject {
  data:
    | {
        data: RustShip;
        type: "RustShip";
      }
    | {
        data: Agent;
        type: "MyAgent";
      };
}

export default WsObject;
