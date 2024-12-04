import { DbAgent } from "./Agent";
import RustShip from "./ship";

interface WsObject {
  data:
    | {
        data: RustShip;
        type: "RustShip";
      }
    | {
        data: DbAgent;
        type: "MyAgent";
      };
}

export default WsObject;
