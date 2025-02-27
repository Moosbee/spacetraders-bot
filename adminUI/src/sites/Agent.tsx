import {
  Button,
  Descriptions,
  DescriptionsProps,
  Divider,
  Flex,
  Space,
} from "antd";
import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import {
  CartesianGrid,
  Legend,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import { DbAgent } from "../models/Agent";
import { backendUrl } from "../store";

function Agent() {
  const { agentID } = useParams();
  const [agent, setAgent] = useState<DbAgent | null>(null);

  const [agentHistory, setAgentHistory] = useState<
    (DbAgent & { datetime: Date })[] | null
  >(null);

  const items: DescriptionsProps["items"] = [
    {
      label: "Symbol",
      key: "symbol",
      children: agent?.symbol,
    },

    {
      label: "Credits",
      key: "credits",
      children: <MoneyDisplay amount={agent?.credits || 0} />,
    },
    {
      label: "Starting Faction",
      key: "startingFaction",
      children: agent?.starting_faction,
    },
    {
      label: "Headquarters",
      key: "headquarters",
      children: agent?.headquarters,
    },
    {
      label: "Ship Count",
      key: "shipCount",
      children: agent?.ship_count,
    },
    {
      label: "Last Updated",
      key: "createdAt",
      children: new Date(agent?.created_at || "").toLocaleString(),
    },
    {
      label: "Account ID",
      key: "accountId",
      children: agent?.account_id || "N/A",
    },
  ];

  useEffect(() => {
    fetch(`http://${backendUrl}/agents/${agentID}`)
      .then((response) => response.json())
      .then((data) => {
        console.log("agent", data);

        setAgent(data);
      });
    fetch(`http://${backendUrl}/agents/${agentID}/history`)
      .then((response) => response.json())
      .then((data) => {
        console.log("agents", data.length);

        const formattedData = (data as DbAgent[]).map((a) => ({
          ...a,
          datetime: new Date(a.created_at),
        }));

        formattedData.sort(
          (a, b) => a.datetime.getTime() - b.datetime.getTime()
        );

        setAgentHistory(formattedData);
      });
  }, [agentID]);

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title={`Agent ${agent?.symbol}`} />
      <Flex gap={12} align="center" justify="space-between">
        <Space>
          <h1>Agent {agent?.symbol}</h1>
          <Button
            onClick={() => {
              fetch(`http://${backendUrl}/agents/${agentID}`)
                .then((response) => response.json())
                .then((data) => {
                  console.log("agent", data);
                  setAgent(data);
                });
              fetch(`http://${backendUrl}/agents/${agentID}/history`)
                .then((response) => response.json())
                .then((data) => {
                  console.log("agents", data.length);

                  const formattedData = (data as DbAgent[]).map((a) => ({
                    ...a,
                    datetime: new Date(a.created_at),
                  }));

                  formattedData.sort(
                    (a, b) => a.datetime.getTime() - b.datetime.getTime()
                  );

                  setAgentHistory(formattedData);
                });
            }}
          >
            Refresh
          </Button>
        </Space>
        <Descriptions bordered column={4} size="small" items={items} />
      </Flex>
      <Divider />
      <div style={{ width: "100%", aspectRatio: "16/6" }}>
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={agentHistory || []} margin={{ top: 5, right: 5 }}>
            <CartesianGrid />
            <XAxis
              dataKey="datetime"
              tickFormatter={(v) => new Date(v).toLocaleString()}
              type="category"
            />
            <YAxis />
            <Tooltip
              content={(props) => {
                if (!props.payload || !props.payload.length || !props.label)
                  return null;
                return (
                  <span>
                    {new Date(props.label).toLocaleString()} :{" "}
                    <MoneyDisplay
                      amount={(props.payload[0].value || 0) as number}
                    />
                  </span>
                );
              }}
            />
            <Legend />
            <Line
              type="monotone"
              dataKey="credits"
              stroke="#8884d8"
              dot={false}
            />
            {/* <Line
              type="monotone"
              dataKey="ship_count"
              stroke="#82ca9d"
              dot={false}
            /> */}
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}

export default Agent;
