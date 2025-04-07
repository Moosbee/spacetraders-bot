import {
  Button,
  Descriptions,
  DescriptionsProps,
  Divider,
  Progress,
  Space,
} from "antd";
import { useEffect, useState } from "react";
import PageTitle from "../features/PageTitle";
import { Info } from "../models/MainInfo";
import useMyStore, { backendUrl } from "../store";

function Main() {
  const reset = useMyStore((state) => state.reset);

  const [data, setData] = useState<Info>({
    agent_symbol: "",
    headquarters: "",
    next_reset_date: "",
    reset_date: "",
    starting_faction: "",
    version: "",
  });

  useEffect(() => {
    fetch(`http://${backendUrl}/insights/run/info`)
      .then((response) => response.json())
      .then((response) => {
        console.log(response);
        setData(response);
      });
  }, []);

  const desc: DescriptionsProps["items"] = [
    {
      label: "Agent Symbol",
      children: data.agent_symbol,
    },
    {
      label: "Headquarters",
      children: data.headquarters,
    },
    {
      label: "Starting Faction",
      children: data.starting_faction,
    },
    {
      label: "Version",
      children: data.version,
    },
    {
      label: "Reset Date",
      children: new Date(data.reset_date).toLocaleString(),
    },
    {
      label: "Next Reset Date",
      children: new Date(data.next_reset_date).toLocaleString(),
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="Main" />
      <h1>Main</h1>
      <Divider />
      <Descriptions column={2} items={desc} />
      <ShipNavProgress
        start_time={data.reset_date}
        end_time={data.next_reset_date}
      />
      <Divider />
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

function ShipNavProgress({
  start_time,
  end_time,
}: {
  start_time: string;
  end_time: string;
}) {
  const [percent, setPercent] = useState(0);

  useEffect(() => {
    const interval = setInterval(() => {
      setPercent(
        Math.round(
          ((new Date().getTime() - new Date(start_time).getTime()) /
            (new Date(end_time).getTime() - new Date(start_time).getTime())) *
            10000
        ) / 100
      );
    }, 1000);

    return () => clearInterval(interval);
  }, [end_time, start_time]);

  return (
    <Progress
      percent={percent}
      size="small"
      format={(value) => `${value?.toFixed(2)}%`}
    />
  );
}

export default Main;
