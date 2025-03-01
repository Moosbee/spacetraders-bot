import {
  Button,
  DatePicker,
  Descriptions,
  DescriptionsProps,
  Divider,
  Flex,
  InputNumber,
  Space,
} from "antd";
import dayjs, { Dayjs } from "dayjs";
import { useEffect, useMemo, useState } from "react";
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

const { RangePicker } = DatePicker;

function calcAgentHistory(
  history: (DbAgent & { datetime: Date })[],
  historyPoints: number,
  minDate: Date,
  maxDate: Date
): {
  symbol: string;
  credits: number;
  shipCount: number;
  datetime: Date;
}[] {
  let current = 0;
  if (history.length === 0) return [];
  const newHistory: {
    symbol: string;
    credits: number;
    shipCount: number;
    datetime: Date;
  }[] = [];
  for (let i = 0; i < historyPoints; i++) {
    const date = calcDate(historyPoints, i, minDate, maxDate);
    while (current < history.length - 1 && history[current].datetime < date) {
      current++;
    }
    const last = history[current - 1] || history[current];
    newHistory.push({
      symbol: last.symbol,
      credits: last.credits,
      shipCount: last.ship_count,
      datetime: date,
    });
  }
  return newHistory;
}

function calcDate(
  notches: number,
  notch: number,
  minDate: Date,
  maxDate: Date
): Date {
  // Validate inputs
  if (notches <= 0) {
    throw new Error("notches must be greater than 0");
  }

  if (notch < 0 || notch >= notches) {
    throw new Error(`notch must be between 0 and ${notches - 1}`);
  }

  // if (minDate > maxDate) {
  //   throw new Error(
  //     `minDate must be less than or equal to maxDate (${minDate} > ${maxDate})`
  //   );
  // }

  // Calculate the total time range in milliseconds
  const timeRange = maxDate.getTime() - minDate.getTime();

  // Calculate the time per notch
  const timePerNotch = timeRange / (notches - 1);

  // Calculate the time for the current notch
  const notchTime = minDate.getTime() + timePerNotch * notch;

  // Return a new Date object for the calculated time
  return new Date(notchTime);
}

function Agent() {
  const { agentID } = useParams();
  const [agent, setAgent] = useState<DbAgent | null>(null);

  const [historyPoints, setHistoryPoints] = useState<number>(1000);

  const [agentHistory, setAgentHistory] = useState<
    (DbAgent & { datetime: Date })[] | null
  >(null);

  const { minDate, maxDate, minDateDayJs, maxDateDayJs } = useMemo(() => {
    const { minDate, maxDate } = agentHistory?.reduce(
      (prev, curr) => {
        return {
          minDate: prev.minDate < curr.datetime ? prev.minDate : curr.datetime,
          maxDate: prev.maxDate > curr.datetime ? prev.maxDate : curr.datetime,
        };
      },
      {
        minDate: new Date(Date.now() - 0),
        maxDate: new Date(0),
      }
    ) || { minDate: new Date(), maxDate: new Date() };
    const minDateDayJs = dayjs(minDate);
    const maxDateDayJs = dayjs(maxDate);

    return {
      minDate,
      maxDate,
      minDateDayJs,
      maxDateDayJs,
    };
  }, [agentHistory]);

  const [dateRange, setDateRange] = useState<[Dayjs, Dayjs]>([
    minDateDayJs,
    maxDateDayJs,
  ]);

  const agentHistoryFiltered = useMemo(() => {
    const [minDateJs, maxDateJs] = dateRange;
    const min = new Date(
      Math.max(minDate.getTime(), minDateJs.toDate().getTime())
    );
    const max = new Date(
      Math.min(maxDate.getTime(), maxDateJs.toDate().getTime())
    );
    return calcAgentHistory(agentHistory || [], historyPoints, min, max);
  }, [dateRange, minDate, maxDate, agentHistory, historyPoints]);

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
          datetime: new Date(a.created_at + "Z"),
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
          <h1>
            Agent {agent?.symbol} {agentHistory?.length}
          </h1>
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
                    datetime: new Date(a.created_at + "Z"),
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
      <Flex justify="space-between" align="center" style={{ marginBottom: 12 }}>
        <span>
          From {minDate.toLocaleString()} to {maxDate.toLocaleString()}
        </span>
        <Space>
          <RangePicker
            showTime
            minDate={minDateDayJs}
            maxDate={maxDateDayJs}
            value={dateRange}
            onChange={(v) => {
              if (v && v[0] && v[1]) {
                setDateRange([v[0], v[1]]);
              }
            }}
          />
          <InputNumber
            min={1}
            max={10000}
            defaultValue={1000}
            value={historyPoints}
            onChange={(v) => setHistoryPoints(v || 1000)}
            step={100}
            changeOnWheel
          />
        </Space>
        <span>
          From{" "}
          {new Date(
            Math.max(minDate.getTime(), dateRange[0].toDate().getTime())
          ).toLocaleString()}{" "}
          to{" "}
          {new Date(
            Math.min(maxDate.getTime(), dateRange[1].toDate().getTime())
          ).toLocaleString()}
        </span>
      </Flex>
      <div style={{ width: "100%", aspectRatio: "16/6" }}>
        <ResponsiveContainer width="100%" height="100%">
          <LineChart
            data={agentHistoryFiltered || []}
            margin={{ left: 30, top: 5, right: 5 }}
          >
            <CartesianGrid />
            <XAxis
              dataKey="datetime"
              tickFormatter={(v) => new Date(v).toLocaleString()}
              type="category"
            />
            <YAxis
              type="number"
              // eslint-disable-next-line @typescript-eslint/no-unused-vars
              tickFormatter={(v, _index) => {
                // console.log("v", v, index);
                return `${(v as number).toLocaleString()}$`;
              }}
              yAxisId="left"
            />
            <YAxis type="number" yAxisId="right" orientation="right" />
            <Tooltip
              content={(props) => {
                if (!props.payload || !props.payload.length || !props.label)
                  return null;
                return (
                  <span>
                    {new Date(props.label).toLocaleString()} :{" "}
                    <MoneyDisplay
                      amount={(props.payload[0].value || 0) as number}
                    />{" "}
                    ({(props.payload[1].value || 0) as number})
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
              yAxisId="left"
            />
            <Line
              type="monotone"
              dataKey="shipCount"
              stroke="#82ca9d"
              dot={false}
              yAxisId="right"
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}

export default Agent;
