import { useQuery } from "@apollo/client/react";
import {
  Button,
  Card,
  Col,
  Divider,
  Flex,
  Popconfirm,
  Progress,
  Row,
  Space,
  Spin,
  Statistic,
  theme,
} from "antd";
import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import {
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  Legend,
  Pie,
  PieChart,
  Tooltip as RechartsTooltip,
  ResponsiveContainer,
  XAxis,
  YAxis,
} from "recharts";
import { backendUrl } from "../data";
import PageTitle from "../features/PageTitle";
import TextLoop from "../features/TextLoop/TextLoop";
import { GET_MAIN_SITE_DATA } from "../graphql/queries";
import { chartColors } from "../utils/chartColors";

function Main() {
  const { loading, error, data, dataState, refetch } = useQuery(
    GET_MAIN_SITE_DATA
    // { pollInterval: 3600000 }
  );
  const {
    token: { colorInfo, geekblue8, green8 },
  } = theme.useToken();

  if (dataState != "complete") return <p>Loading...</p>;
  if (error) return <p>Error: {error.message}</p>;

  const fleetsByType: {
    [key: string]: {
      fleetCount: number;
      inactiveFleetCount: number;
      assignmentCount: number;
      openAssignmentCount: number;
    };
  } = {};
  data.fleets.forEach((fleet) => {
    if (!fleetsByType[fleet.fleetType]) {
      fleetsByType[fleet.fleetType] = {
        fleetCount: 0,
        assignmentCount: 0,
        inactiveFleetCount: 0,
        openAssignmentCount: 0,
      };
    }
    fleetsByType[fleet.fleetType].fleetCount++;
    if (!fleet.active) {
      fleetsByType[fleet.fleetType].inactiveFleetCount++;
    }
    fleetsByType[fleet.fleetType].assignmentCount += fleet.assignments.length;
    fleetsByType[fleet.fleetType].openAssignmentCount +=
      fleet.assignments.filter((f) => !f.ship).length;
  });

  const fleets = Object.entries(fleetsByType)
    .sort((a, b) => a[0].localeCompare(b[0]))
    .map(([key, value]) => ({
      name: key,
      fleetCount: value.fleetCount,
      assignmentCount: value.assignmentCount,
      activeFleetCount: value.inactiveFleetCount,
      openAssignmentCount: value.openAssignmentCount,
    }));

  type shipStatusType =
    (typeof data.ships)[number]["status"]["status"]["__typename"];

  const shipStatus = Object.entries(
    data.ships
      .map((ship) => ship.status.status.__typename)
      .reduce((ob, ship) => {
        ob[ship] = (ob[ship] || 0) + 1;
        return ob;
      }, {} as Record<shipStatusType, number>)
  )
    .map(([type, count]) => {
      return {
        fleetType: type.replace("Status", ""),
        count: count,
      };
    })
    .sort((a, b) => a.fleetType.localeCompare(b.fleetType))
    .reverse();
  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="Main" />
      <Spin spinning={loading}>
        <ShipNavProgress
          start_time={data.runInfo.resetDate}
          end_time={data.runInfo.nextResetDate}
        />
        <Divider />
        <Row gutter={16}>
          <Col span={10}>
            <Flex gap={16} vertical>
              <Row gutter={16}>
                <Col span={12}>
                  <Card variant="borderless">
                    <Row gutter={16}>
                      <Col span={12}>
                        <Statistic
                          title="Symbol"
                          value={data.runInfo.agent?.symbol}
                        />
                        <Statistic
                          title="API Backlog Count"
                          value={data.apiCounts}
                        />
                      </Col>
                      <Col span={12}>
                        <Statistic
                          title="Credits"
                          value={data.runInfo.agent?.credits}
                        />
                        <Statistic
                          title="Reserved Credits"
                          value={data.budget.reservedAmount}
                        />
                      </Col>
                    </Row>
                  </Card>
                </Col>
                <Col span={12}>
                  <Card variant="borderless">
                    <Row gutter={16}>
                      <Col span={6}>
                        <Statistic
                          valueStyle={{
                            color:
                              data.chartManager.channelState.state === "CLOSED"
                                ? "red"
                                : "currentColor",
                          }}
                          title="Chart"
                          value={data.chartManager.channelState.usedCapacity}
                          suffix={
                            <Spin
                              spinning={data.chartManager.busy}
                              size="small"
                            />
                          }
                        />
                      </Col>
                      <Col span={6}>
                        <Statistic
                          valueStyle={{
                            color:
                              data.fleetManager.channelState.state === "CLOSED"
                                ? "red"
                                : "currentColor",
                          }}
                          title="Fleet"
                          value={data.fleetManager.channelState.usedCapacity}
                          suffix={
                            <Spin
                              spinning={data.fleetManager.busy}
                              size="small"
                            />
                          }
                        />
                      </Col>
                      <Col span={6}>
                        <Statistic
                          valueStyle={{
                            color:
                              data.tradeManager.channelState.state === "CLOSED"
                                ? "red"
                                : "currentColor",
                          }}
                          title="Trade"
                          value={data.tradeManager.channelState.usedCapacity}
                          suffix={
                            <Spin
                              spinning={data.tradeManager.busy}
                              size="small"
                            />
                          }
                        />
                      </Col>
                      <Col span={6}>
                        <Statistic
                          valueStyle={{
                            color:
                              data.miningManager.channelState.state === "CLOSED"
                                ? "red"
                                : "currentColor",
                          }}
                          title="Mining"
                          value={data.miningManager.channelState.usedCapacity}
                          suffix={
                            <Spin
                              spinning={data.miningManager.busy}
                              size="small"
                            />
                          }
                        />
                      </Col>
                      <Col span={6}>
                        <Statistic
                          valueStyle={{
                            color:
                              data.contractManager.channelState.state ===
                              "CLOSED"
                                ? "red"
                                : "currentColor",
                          }}
                          title="Contract"
                          value={data.contractManager.channelState.usedCapacity}
                          suffix={
                            <Spin
                              spinning={data.contractManager.busy}
                              size="small"
                            />
                          }
                        />
                      </Col>
                      <Col span={6}>
                        <Statistic
                          title="Scrapping"
                          valueStyle={{
                            color:
                              data.scrappingManager.channelState.state ===
                              "CLOSED"
                                ? "red"
                                : "currentColor",
                          }}
                          value={
                            data.scrappingManager.channelState.usedCapacity
                          }
                          suffix={
                            <Spin
                              spinning={data.scrappingManager.busy}
                              size="small"
                            />
                          }
                        />
                      </Col>
                      <Col span={6}>
                        <Statistic
                          title="Construction"
                          valueStyle={{
                            color:
                              data.constructionManager.channelState.state ===
                              "CLOSED"
                                ? "red"
                                : "currentColor",
                          }}
                          value={
                            data.constructionManager.channelState.usedCapacity
                          }
                          suffix={
                            <Spin
                              spinning={data.constructionManager.busy}
                              size="small"
                            />
                          }
                        />
                      </Col>
                    </Row>
                  </Card>
                </Col>
              </Row>
              <Card variant="borderless">
                <div className="overflow-hidden w-full">
                  <div className="flex justify-around gap-4">
                    {data.systems.length <= 6 ? (
                      <>
                        {data.systems.map((system) => (
                          <span key={system.symbol}>
                            <Link to={`system/${system.symbol}`}>
                              {system.symbol}
                            </Link>
                          </span>
                        ))}
                      </>
                    ) : (
                      <>
                        <TextLoop
                          texts={data.systems.map((system) => (
                            <Link to={`system/${system.symbol}`}>
                              {system.symbol}
                            </Link>
                          ))}
                          duration={data.systems.length * 1}
                        />
                      </>
                    )}
                  </div>
                </div>
                <div className="overflow-hidden w-full">
                  <ul className="flex justify-around gap-4">
                    {data.fleets.length <= 4 ? (
                      <>
                        {data.fleets.map((fleet) => (
                          <li key={fleet.id} className="whitespace-nowrap">
                            {fleet.fleetType}-{fleet.systemSymbol}
                          </li>
                        ))}
                      </>
                    ) : (
                      <>
                        <TextLoop
                          texts={data.fleets.map(
                            (fleet) =>
                              `${fleet.fleetType}_${fleet.id}_${fleet.systemSymbol}`
                          )}
                          duration={data.fleets.length * 3}
                        />
                      </>
                    )}
                  </ul>
                </div>
                <div className="overflow-hidden w-full">
                  <ul className="">
                    {data.ships.length <= 5 ? (
                      <>
                        {data.ships.map((ship) => (
                          <li key={ship.symbol} className="whitespace-nowrap">
                            {ship.symbol}
                          </li>
                        ))}
                      </>
                    ) : (
                      <>
                        <TextLoop
                          texts={data.ships.map((ship) => ship.symbol)}
                          duration={data.ships.length * 1.5}
                          direction="left-to-right"
                        />
                      </>
                    )}
                  </ul>
                </div>
              </Card>
            </Flex>
          </Col>
          <Col span={6}>
            <Card variant="borderless">
              <Row>
                <Col span={12}>
                  <Statistic
                    title="Occupied Systems"
                    value={data.systems.length}
                  />
                </Col>
                <Col span={12}>
                  <Statistic
                    title="Systems with Ships"
                    value={
                      data.systems.filter((s) =>
                        data.ships.some(
                          (ship) => ship.nav?.systemSymbol === s.symbol
                        )
                      ).length
                    }
                  />
                </Col>
              </Row>
              <Row>
                <Col span={12}>
                  <Statistic
                    title="Total Waypoints"
                    value={data.systems
                      .map((f) => f.waypoints.length)
                      .reduce((total, current) => total + current)}
                  />
                </Col>
                <Col span={12}>
                  <Statistic
                    title="Uncharted Waypoints"
                    value={data.systems
                      .map(
                        (f) => f.waypoints.filter((w) => !w.chartedBy).length
                      )
                      .reduce((total, current) => total + current)}
                  />
                </Col>
              </Row>
              <Row>
                <Col span={12}>
                  <Statistic
                    title="Total Marketplaces"
                    value={data.systems
                      .map(
                        (f) =>
                          f.waypoints.filter((m) => m.hasMarketplace).length
                      )
                      .reduce((total, current) => total + current)}
                  />
                </Col>
                <Col span={12}>
                  <Statistic
                    title="Uncharted Marketplaces"
                    value={data.systems
                      .map(
                        (f) =>
                          f.waypoints
                            .filter((w) => !w.chartedBy)
                            .filter((m) => m.hasMarketplace).length
                      )
                      .reduce((total, current) => total + current)}
                  />
                </Col>
              </Row>
              <Row>
                <Col span={12}>
                  <Statistic
                    title="Total Shipyards"
                    value={data.systems
                      .map(
                        (f) => f.waypoints.filter((m) => m.hasShipyard).length
                      )
                      .reduce((total, current) => total + current)}
                  />
                </Col>
                <Col span={12}>
                  <Statistic
                    title="Uncharted Shipyards"
                    value={data.systems
                      .map(
                        (f) =>
                          f.waypoints
                            .filter((w) => !w.chartedBy)
                            .filter((m) => m.hasShipyard).length
                      )
                      .reduce((total, current) => total + current)}
                  />
                </Col>
              </Row>
            </Card>
          </Col>
          <Col span={8}>
            <Card variant="borderless">
              <Row>
                <Col span={8}>
                  <Row>
                    <Col span={24}>
                      <Statistic
                        title="Total Ships"
                        value={data.ships.length}
                      />
                    </Col>
                    <Col span={24}>
                      <Statistic
                        title="Ships no Assignment"
                        value={
                          data.ships.filter((s) => {
                            return (
                              !s.status ||
                              (s.status.assignmentId === null &&
                                s.status.tempAssignmentId === null)
                            );
                          }).length
                        }
                      />
                    </Col>
                    <Col span={24}>
                      <Statistic
                        title="Ships in Navigation"
                        value={
                          data.ships.filter(
                            (s) => s.nav?.status === "IN_TRANSIT"
                          ).length
                        }
                      />
                    </Col>
                    <Col span={24}>
                      <Statistic
                        title="Total Cargo Units"
                        value={data.ships
                          .map((s) => s.cargo.units)
                          .reduce((a, b) => a + b, 0)}
                      />
                    </Col>
                  </Row>
                </Col>

                <Col span={16}>
                  <ResponsiveContainer>
                    <PieChart>
                      <Pie
                        data={shipStatus}
                        dataKey="count"
                        nameKey="fleetType"
                        cx="50%"
                        cy="50%"
                        outerRadius="50%"
                        isAnimationActive
                        label={(entry) => `${entry.name}: ${entry.count}`}
                      >
                        {shipStatus.map((_, index) => (
                          <Cell
                            key={`cell-${index}`}
                            fill={chartColors[index % chartColors.length]}
                          />
                        ))}
                      </Pie>
                      <Legend
                        formatter={(value, entry) =>
                          `${value} (${entry.payload?.value})`
                        }
                      />
                    </PieChart>
                  </ResponsiveContainer>
                </Col>
              </Row>
            </Card>
          </Col>
        </Row>
        <Row gutter={16} style={{ marginTop: 16 }}>
          <Col span={10}>
            <Card variant="borderless" title="Construction">
              <div style={{ width: "100%", height: 400 }}>
                <ResponsiveContainer>
                  <BarChart
                    data={
                      data.runInfo.headquartersSystem?.constructionMaterials ||
                      []
                    }
                  >
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="tradeSymbol" />
                    <YAxis scale="linear" />
                    <RechartsTooltip labelStyle={{ color: colorInfo }} />
                    <Legend />
                    <Bar
                      dataKey="required"
                      fill={geekblue8}
                      isAnimationActive
                    />
                    <Bar dataKey="fulfilled" fill={green8} isAnimationActive />
                  </BarChart>
                </ResponsiveContainer>
              </div>
            </Card>
          </Col>
          <Col span={14}>
            <Card variant="borderless" title="Fleets by Type">
              <div className="flex justify-between">
                <div>
                  <Statistic title="Total Fleets" value={data.fleets.length} />
                  <Statistic
                    title="Total Assignments"
                    value={data.fleets
                      .map((f) => f.assignments.length)
                      .reduce((prev, now) => prev + now)}
                  />
                  <Statistic
                    title="Open Assignments"
                    value={data.shipAssignments.length}
                  />
                </div>

                <div style={{ width: "80%", height: 400 }} className="">
                  <ResponsiveContainer>
                    <PieChart>
                      <Pie
                        data={fleets}
                        dataKey="fleetCount"
                        nameKey="name"
                        cx="50%"
                        cy="50%"
                        outerRadius="40%"
                        isAnimationActive
                        // label={(entry) => `${entry.name}: ${entry.value}`}
                      >
                        {Object.entries(fleetsByType)
                          .sort((a, b) => a[0].localeCompare(b[0]))
                          .map((_, index) => (
                            <Cell
                              key={`cell-${index}`}
                              fill={chartColors[index % chartColors.length]}
                            />
                          ))}
                      </Pie>
                      <Pie
                        data={fleets}
                        dataKey="assignmentCount"
                        nameKey="name"
                        cx="50%"
                        cy="50%"
                        outerRadius="80%"
                        innerRadius="60%"
                        isAnimationActive
                        label={(entry) =>
                          `${entry.name}: ${entry.fleetCount} - ${entry.assignmentCount}`
                        }
                      >
                        {Object.entries(fleetsByType)
                          .sort((a, b) => a[0].localeCompare(b[0]))
                          .map((_, index) => (
                            <Cell
                              key={`cell-${index}`}
                              fill={chartColors[index % chartColors.length]}
                            />
                          ))}
                      </Pie>
                      <Legend
                        content={() => {
                          return (
                            <div>
                              <ul className="flex pt-5 gap-2 justify-around">
                                {Object.entries(fleetsByType)
                                  .sort((a, b) => a[0].localeCompare(b[0]))
                                  .map(([key], index) => (
                                    <li
                                      key={index}
                                      style={{
                                        color:
                                          chartColors[
                                            index % chartColors.length
                                          ],
                                      }}
                                      className="w-fit"
                                    >
                                      {key}
                                      <br />
                                      Fleets: {fleetsByType[key].fleetCount} (
                                      {fleetsByType[key].inactiveFleetCount})
                                      <br />
                                      ASGMT: {fleetsByType[key].assignmentCount}
                                      <br />
                                      Open ASGMT:{" "}
                                      {fleetsByType[key].openAssignmentCount}
                                    </li>
                                  ))}
                              </ul>
                            </div>
                          );
                        }}
                      />
                    </PieChart>
                  </ResponsiveContainer>
                </div>
              </div>
            </Card>
          </Col>
        </Row>
        <Divider />
        <Space>
          <Button
            onClick={() => {
              refetch();
            }}
          >
            Reload
          </Button>
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
          <Popconfirm
            title="Do you want to Reset the App?"
            description={
              <span>
                Please make sure all other tabs are closed!
                <br /> So that this is the only open Tap here.
                <br /> This will delete all data from IndexedDB and reload the
                application.
              </span>
            }
            onConfirm={() => {
              indexedDB.deleteDatabase("myApp");
              window.location.reload();
            }}
            okText="OK"
            cancelText="No"
          >
            <Button danger>Clear Everything</Button>
          </Popconfirm>
        </Space>
      </Spin>
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
    <div className="">
      <Progress
        percent={percent}
        size="small"
        format={(value) => `${value?.toFixed(2)}%`}
      />
      <div className="flex justify-between">
        <span>{new Date(start_time).toLocaleString()}</span>
        <span>{new Date(end_time).toLocaleString()}</span>
      </div>
    </div>
  );
}

export default Main;
