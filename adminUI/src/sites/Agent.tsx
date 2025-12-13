import { useQuery } from "@apollo/client/react";
import {
  Button,
  Descriptions,
  DescriptionsProps,
  Divider,
  Flex,
  Space,
  Spin,
} from "antd";
import { useParams } from "react-router-dom";
import { AgentHistoryGraph } from "../features/AgentHistoryGraph/AgentHistoryGraph";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import { GET_AGENT_HISTORY } from "../graphql/queries";

function Agent() {
  const { agentID } = useParams();

  const { loading, error, data, dataState, refetch } = useQuery(
    GET_AGENT_HISTORY,
    {
      variables: {
        agentSymbol: agentID || "",
      },
    }
  );

  if (dataState != "complete") return <p>Loading...</p>;
  if (error) return <p>Error: {error.message}</p>;

  const agent = data.agent;

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
      children: agent?.startingFaction,
    },
    {
      label: "Headquarters",
      key: "headquarters",
      children: agent?.headquarters,
    },
    {
      label: "Ship Count",
      key: "shipCount",
      children: agent?.shipCount,
    },
    {
      label: "Last Updated",
      key: "createdAt",
      children: new Date(agent?.createdAt || "").toLocaleString(),
    },
    {
      label: "Account ID",
      key: "accountId",
      children: agent?.accountId || "N/A",
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title={`Agent ${agent?.symbol}`} />
      <Flex gap={12} align="center" justify="space-between">
        <Space>
          <h1 className="scroll-m-20 text-center text-3xl font-bold tracking-tight text-balance">
            Agent {agent?.symbol} {agent.history?.length}
          </h1>
          <Button
            onClick={() => {
              refetch();
            }}
          >
            Refresh
          </Button>
        </Space>
        <Spin spinning={loading}>
          <Descriptions bordered column={4} size="small" items={items} />
        </Spin>
      </Flex>
      <Divider />
      <AgentHistoryGraph
        agentHistory={data.agent.history.map((ag) => {
          return {
            ...ag,
            datetime: new Date(ag.createdAt),
          };
        })}
      />
    </div>
  );
}

export default Agent;
