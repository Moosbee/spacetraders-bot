import { Button, Space } from "antd";
import useMyStore, { backendUrl } from "../store";

function Main() {
  const reset = useMyStore((state) => state.reset);

  return (
    <div>
      <h1>main</h1>
      <Space>
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
        <Button onClick={reset}>Reset Client State</Button>
      </Space>
    </div>
  );
}

export default Main;
