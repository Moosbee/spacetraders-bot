import {
  Checkbox,
  Col,
  DatePicker,
  Divider,
  Flex,
  InputNumber,
  Row,
  Space,
  theme,
} from "antd";
import { CheckboxGroupProps } from "antd/es/checkbox";
import dayjs, { Dayjs } from "dayjs";
import { useMemo, useState } from "react";
import {
  CartesianGrid,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { TradeSymbol } from "../../models/api";
import { MarketTradeGood } from "../../models/Market";
import { chartColors } from "../../utils/chartColors";
import { cyrb53 } from "../../utils/utils";
const { RangePicker } = DatePicker;

function calcTradeHistory(
  history: { [key in TradeSymbol]?: (MarketTradeGood & { datetime: Date })[] },
  historyPoints: number,
  minDate: Date,
  maxDate: Date,
  filterValues: TradeSymbol[]
): {
  datetime: Date;
  values: { [key in TradeSymbol]?: MarketTradeGood & { datetime: Date } };
}[] {
  // If there are no entries in any history array, return empty array
  if (Object.values(history).every((arr) => !arr || arr.length === 0)) {
    return [];
  }

  const result = [];

  // For each history point, calculate the date and find the most recent value for each symbol
  for (let i = 0; i < historyPoints; i++) {
    const date = calcDate(historyPoints, i, minDate, maxDate);
    const values: { [key: string]: MarketTradeGood & { datetime: Date } } = {};

    // Process each symbol's history array
    for (const symbolStr in history) {
      const symbol = symbolStr as TradeSymbol;
      if (!filterValues.includes(symbol)) continue;
      const symbolHistory = history[symbol];
      if (!symbolHistory || symbolHistory.length === 0) continue;

      // Find the most recent entry before the current date point
      let currentIndex = 0;
      while (
        currentIndex < symbolHistory.length - 1 &&
        symbolHistory[currentIndex + 1].datetime.getTime() <= date.getTime()
      ) {
        currentIndex++;
      }

      // Use the found entry
      if (symbolHistory[currentIndex].datetime <= date) {
        values[symbol] = symbolHistory[currentIndex];
      }

      // console.log(
      //   "calcTradeHistory",
      //   date,
      //   symbol,
      //   history[symbol],
      //   currentIndex,
      //   values
      // );
    }

    // Add this point to the result
    result.push({
      datetime: date,
      values: values,
    });
  }

  return result;
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

function MarketTradeHistory({ history }: { history: MarketTradeGood[] }) {
  // const trade_history: Record<string, MarketTradeGood[]> = {};

  const {
    token: { colorBgElevated },
  } = theme.useToken();

  const tradeHistory = useMemo(() => {
    const trade_history: {
      [key in TradeSymbol]?: (MarketTradeGood & { datetime: Date })[];
    } = {};

    for (let i = 0; i < history.length; i++) {
      const trade = history[i];
      const new_trade = { ...trade, datetime: new Date(trade.created_at) };
      const good = trade_history[trade.symbol] || [];
      good.push(new_trade);
      trade_history[trade.symbol] = good;
    }

    for (const symbol in trade_history) {
      trade_history[symbol as TradeSymbol]?.sort(
        (a, b) => a.datetime.getTime() - b.datetime.getTime()
      );
    }

    return trade_history;
  }, [history]);

  const { minDate, maxDate, minDateDayJs, maxDateDayJs } = useMemo(() => {
    const { minDate, maxDate } = Object.values(tradeHistory)
      ?.flat()
      ?.reduce(
        (prev, curr) => {
          return {
            minDate:
              prev.minDate < curr.datetime ? prev.minDate : curr.datetime,
            maxDate:
              prev.maxDate > curr.datetime ? prev.maxDate : curr.datetime,
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
  }, [tradeHistory]);

  const [dateRange, setDateRange] = useState<[Dayjs, Dayjs]>([
    minDateDayJs,
    maxDateDayJs,
  ]);

  const [historyPoints, setHistoryPoints] = useState<number>(1000);

  const [checkboxValue, setCheckboxValue] = useState<TradeSymbol[]>([]);

  const chartHistory = useMemo(() => {
    const chtHist = calcTradeHistory(
      tradeHistory,
      historyPoints,
      minDate,
      maxDate,
      checkboxValue
    );
    console.log("chtHist", chtHist, tradeHistory);
    return chtHist;
  }, [checkboxValue, historyPoints, maxDate, minDate, tradeHistory]);

  const colors = useMemo(() => {
    const colors: Record<string, string> = {};
    for (const symbol in tradeHistory) {
      colors[symbol] = chartColors[cyrb53(symbol, 8888) % chartColors.length];
    }
    return colors;
  }, [tradeHistory]);

  const checkboxValues: CheckboxGroupProps["options"] = useMemo(() => {
    const value: CheckboxGroupProps["options"] = [];
    for (const symbol in tradeHistory) {
      value.push({
        label: symbol,
        value: symbol,
        style: {
          color: colors[symbol],
        },
      });
    }
    return value;
  }, [colors, tradeHistory]);

  return (
    <div>
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
      <Divider />
      <Row>
        <Col span={2}>
          <Checkbox.Group
            options={checkboxValues}
            // defaultValue={["Apple"]}
            // onChange={(v) => console.log("v", v)}
            value={checkboxValue}
            onChange={(v) => {
              setCheckboxValue(v);
            }}
          />
        </Col>
        <Col span={22}>
          <div style={{ width: "100%", aspectRatio: "16/6" }}>
            <ResponsiveContainer width="100%" height="100%">
              <LineChart
                data={chartHistory || []}
                margin={{ left: 30, top: 5, right: 5 }}
              >
                <CartesianGrid />
                <Tooltip
                  content={(props) => {
                    if (
                      !props.payload ||
                      !props.payload.length ||
                      !props.label ||
                      props.payload.length % 3 !== 0
                    )
                      return null;
                    console.log("props", props);
                    const values = props.payload
                      .filter((p) => p.color && p.name && p.value)
                      .map((p) => ({
                        color: p.color as string,
                        value: p.value as number,
                        name: p.name as string,
                      }));
                    values.sort((a, b) => b.name.localeCompare(a.name));

                    const text: {
                      name: string;
                      color: string;
                      sellPrice: number;
                      purchasePrice: number;
                      tradeVolume: number;
                    }[] = [];
                    for (let i = 0; i < values.length; i = i + 3) {
                      text.push({
                        name: values[i].name.split(" ")[0],
                        color: values[i].color,
                        sellPrice: values[i + 2].value,
                        purchasePrice: values[i + 1].value,
                        tradeVolume: values[i].value,
                      });
                    }
                    return (
                      <div
                        style={{
                          whiteSpace: "pre-wrap",
                          background: colorBgElevated,
                          padding: "0.5rem",
                        }}
                      >
                        {new Date(props.label).toLocaleString()}:
                        <br />
                        {text.map((t, i) => (
                          <span key={i}>
                            <span style={{ color: t.color }}>
                              {t.name}: {t.sellPrice} -{">"} {t.purchasePrice} (
                              {t.tradeVolume})
                            </span>
                            <br />
                          </span>
                        ))}
                      </div>
                    );
                  }}
                />
                <XAxis
                  dataKey="datetime"
                  tickFormatter={(v) => new Date(v).toLocaleString()}
                  type="category"
                />
                <YAxis
                  type="number"
                  // domain={["dataMin - 10000", "dataMax + 10000"]}
                  domain={([dataMin, dataMax]) => {
                    const min = Math.floor((dataMin * 0.9) / 100) * 100;
                    // const min = 0;
                    const max = Math.ceil((dataMax * 1.05) / 100) * 100;
                    return [min, max];
                  }}
                  // eslint-disable-next-line @typescript-eslint/no-unused-vars
                  tickFormatter={(v, _index) => {
                    // console.log("v", v, index);
                    return `${(v as number).toLocaleString()}$`;
                  }}
                  yAxisId="left"
                />
                <YAxis type="number" yAxisId="right" orientation="right" />

                {Object.values(checkboxValue).map((symbol) => (
                  <Line
                    key={symbol + " sell"}
                    type="monotone"
                    dataKey={(entry: {
                      datetime: Date;
                      values: {
                        [key in TradeSymbol]?: MarketTradeGood & {
                          datetime: Date;
                        };
                      };
                    }) => {
                      const val =
                        entry.values[symbol as TradeSymbol]?.sell_price || 0;
                      return val;
                    }}
                    name={symbol + " sell"}
                    stroke={colors[symbol]}
                    dot={false}
                    yAxisId="left"
                  />
                ))}

                {Object.values(checkboxValue).map((symbol) => (
                  <Line
                    key={symbol + " buy"}
                    type="monotone"
                    dataKey={(entry: {
                      datetime: Date;
                      values: {
                        [key in TradeSymbol]?: MarketTradeGood & {
                          datetime: Date;
                        };
                      };
                    }) => {
                      const val =
                        entry.values[symbol as TradeSymbol]?.purchase_price ||
                        0;
                      return val;
                    }}
                    name={symbol + " buy"}
                    stroke={colors[symbol]}
                    dot={false}
                    yAxisId="left"
                  />
                ))}
                {Object.values(checkboxValue).map((symbol) => (
                  <Line
                    key={symbol + " volume"}
                    type="monotone"
                    dataKey={(entry: {
                      datetime: Date;
                      values: {
                        [key in TradeSymbol]?: MarketTradeGood & {
                          datetime: Date;
                        };
                      };
                    }) => {
                      const val =
                        entry.values[symbol as TradeSymbol]?.trade_volume || 0;
                      return val;
                    }}
                    name={symbol + " volume"}
                    stroke={colors[symbol]}
                    dot={false}
                    yAxisId="right"
                  />
                ))}

                {/* <Line
              type="monotone"
              dataKey="credits"
              stroke="#8884d8"
              dot={false}
              yAxisId="left"
            /> */}
              </LineChart>
            </ResponsiveContainer>
          </div>
        </Col>
      </Row>
    </div>
  );
}

export default MarketTradeHistory;
