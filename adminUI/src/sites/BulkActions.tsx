import { Button, Flex, Input, Typography } from "antd";
import { useState } from "react";
import PageTitle from "../features/PageTitle";
import { backendUrl } from "../store";

const { TextArea } = Input;
const { Text } = Typography;

function BulkActions() {
  const [text, setText] = useState("");

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="Bulk Actions" />
      <h1>BulkActions</h1>
      <Text>
        Supported:
        <ul>
          <li>navigate ship_symbol -{">"} Waypoint_symbol</li>
        </ul>
      </Text>
      <Flex gap={12} vertical>
        <TextArea
          rows={20}
          onChange={(e) => setText(e.target.value)}
          value={text}
        />
        <Button
          onClick={() => {
            console.log("Complete text", text);

            const lines = text.split("\n");
            for (const line of lines) {
              console.log("lineer", line);
              const args = line.split(" ");
              if (args[0] === "navigate" && args[2] === "->") {
                const ship_symbol = args[1];
                const Waypoint_symbol = args[3];
                fetch(`http://${backendUrl}/ship/${ship_symbol}/navigate`, {
                  method: "POST",
                  headers: { "Content-Type": "application/json" },
                  body: JSON.stringify({ waypointSymbol: Waypoint_symbol }),
                })
                  .then((response) => response.json())
                  .then((data) => console.log("denden", data));
              }
            }
          }}
        >
          Send
        </Button>
      </Flex>
    </div>
  );
}

export default BulkActions;
