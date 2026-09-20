import { useQuery } from "@apollo/client/react";
import { Button, Divider, Empty, Result, Row, Select, Space, Spin } from "antd";
import { useState } from "react";
import PageTitle from "../features/PageTitle";
import SystemCard from "../features/SystemCard/SystemCard";
import { GET_SYSTEMS_WITH_FLEETS_OR_SHIPS } from "../graphql/queries";

function SystemsWithFleets() {
  const { loading, error, data, refetch } = useQuery(
    GET_SYSTEMS_WITH_FLEETS_OR_SHIPS,
  );

  const [mapScale, setMapScale] = useState<
    "2/1" | "1/1" | "1/2" | "1/3" | "1/4" | "1/8" | "1/16"
  >("1/2");
  const [canvasSize, setCanvasSize] = useState<number>(2000);

  if (error) {
    return (
      <Result
        status="error"
        title="Systems Error"
        subTitle={error.message}
        extra={[
          <Button key="retry" type="primary" onClick={() => refetch()}>
            Try Again
          </Button>,
        ]}
      />
    );
  }

  const systems = data?.systems.items ?? [];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="Occupied Systems" />
      <Spin spinning={loading}>
        <Space>
          <h1 className="scroll-m-20 text-center text-3xl font-bold tracking-tight text-balance">
            Occupied Systems
          </h1>
          <Button onClick={() => refetch()}>Refresh</Button>
          <Divider type="vertical" />
          <Space>
            <span>Map Scale:</span>
            <Select
              value={mapScale}
              style={{ width: 100 }}
              onChange={(value) => setMapScale(value)}
              options={["2/1", "1/1", "1/2", "1/3", "1/4", "1/8", "1/16"].map(
                (m) => ({
                  label: m,
                  value: m,
                }),
              )}
            />
            <span>Canvas Size:</span>
            <Select
              value={canvasSize}
              style={{ width: 100 }}
              onChange={(value) => setCanvasSize(value)}
              options={[2000, 4000, 8000, 16000].map((s) => ({
                label: `${s}px`,
                value: s,
              }))}
            />
          </Space>
        </Space>
        <Divider />
        {systems.length === 0 && !loading ? (
          <Empty description="No systems with fleets or ships" />
        ) : (
          <Row gutter={[16, 16]}>
            {systems.map((system) => (
              <SystemCard
                key={system.symbol}
                system={system}
                mapScale={mapScale}
                canvasSize={canvasSize}
              />
            ))}
          </Row>
        )}
      </Spin>
    </div>
  );
}

export default SystemsWithFleets;
