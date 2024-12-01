import { Button } from "antd";
import { backendUrl } from "../store";

function Main() {
  return (
    <div>
      <h1>main</h1>
      <Button
        onClick={() => {
          fetch(`http://${backendUrl}/shutdown`, { method: "POST" }).then(
            (response) => {
              console.log(response);
              alert("shutdown");
            }
          );
        }}
      >
        Shutdown
      </Button>
    </div>
  );
}

export default Main;
