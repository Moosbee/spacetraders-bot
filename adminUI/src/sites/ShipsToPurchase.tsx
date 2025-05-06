import { Button, Flex, List, Space, Spin } from "antd";
import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { backendUrl } from "../data";
import PageTitle from "../features/PageTitle";
import { ShipManagementResponse } from "../models/RequiredShips";

export default function ShipsToPurchase() {
  const [requiredShips, setRequiredShips] =
    useState<ShipManagementResponse | null>();

  const [loading, setLoading] = useState(false);

  useEffect(() => {
    setLoading(true);
    fetch(`http://${backendUrl}/insights/ship/discrepancy`)
      .then((response) => response.json())
      .then((data) => {
        console.log("/insights/ship/discrepancy", data);

        setLoading(false);
        setRequiredShips(data);
      });
  }, []);

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="Ships to Purchase" />
      <Space>
        <h1>
          Ships to Purchase{" "}
          {Object.values(requiredShips?.chart.ships ?? {})
            .map((ship) => ship.length)
            .reduce((a, b) => a + b, 0) +
            Object.values(requiredShips?.construction.ships ?? {})
              .map((ship) => ship.length)
              .reduce((a, b) => a + b, 0) +
            Object.values(requiredShips?.contract.ships ?? {})
              .map((ship) => ship.length)
              .reduce((a, b) => a + b, 0) +
            Object.values(requiredShips?.mining.ships ?? {})
              .map((ship) => ship.length)
              .reduce((a, b) => a + b, 0) +
            Object.values(requiredShips?.scrap.ships ?? {})
              .map((ship) => ship.length)
              .reduce((a, b) => a + b, 0) +
            Object.values(requiredShips?.trading.ships ?? {})
              .map((ship) => ship.length)
              .reduce((a, b) => a + b, 0)}
        </h1>
        <Button
          onClick={() => {
            setLoading(true);
            fetch(`http://${backendUrl}/insights/ship/discrepancy`)
              .then((response) => response.json())
              .then((data) => {
                console.log("/insights/ship/discrepancy", data);

                setLoading(false);
                setRequiredShips(data);
              });
          }}
        >
          Refresh
        </Button>
        <Spin spinning={loading} />
      </Space>
      <Flex gap={24} align="center" justify="center">
        <List
          header="Chart"
          size="small"
          bordered
          dataSource={Object.entries(requiredShips?.chart.ships ?? {})}
          renderItem={(ship) => (
            <List.Item>
              <Link to={`/system/${ship[0]}`}>{ship[0]}</Link>:
              <List
                size="small"
                dataSource={ship[1]}
                renderItem={(ship) => (
                  <List.Item>
                    {ship[0]} {ship[1]} {ship[2]}
                  </List.Item>
                )}
              />
            </List.Item>
          )}
        />
        <List
          header="Construction"
          size="small"
          bordered
          dataSource={Object.entries(requiredShips?.construction.ships ?? {})}
          renderItem={(ship) => (
            <List.Item>
              <Link to={`/system/${ship[0]}`}>{ship[0]}</Link>:
              <List
                size="small"
                dataSource={ship[1]}
                renderItem={(ship) => (
                  <List.Item>
                    {ship[0]} {ship[1]} {ship[2]}
                  </List.Item>
                )}
              />
            </List.Item>
          )}
        />
        <List
          header="Contract"
          size="small"
          bordered
          dataSource={Object.entries(requiredShips?.contract.ships ?? {})}
          renderItem={(ship) => (
            <List.Item>
              <Link to={`/system/${ship[0]}`}>{ship[0]}</Link>:
              <List
                size="small"
                dataSource={ship[1]}
                renderItem={(ship) => (
                  <List.Item>
                    {ship[0]} {ship[1]} {ship[2]}
                  </List.Item>
                )}
              />
            </List.Item>
          )}
        />
        <List
          header="Mining"
          size="small"
          bordered
          dataSource={Object.entries(requiredShips?.mining.ships ?? {})}
          renderItem={(ship) => (
            <List.Item>
              <Link to={`/system/${ship[0]}`}>{ship[0]}</Link>:
              <List
                size="small"
                dataSource={ship[1]}
                renderItem={(ship) => (
                  <List.Item>
                    {ship[0]} {ship[1]} {ship[2]}
                  </List.Item>
                )}
              />
            </List.Item>
          )}
        />
        <List
          header="Scrapping"
          size="small"
          bordered
          dataSource={Object.entries(requiredShips?.scrap.ships ?? {})}
          renderItem={(ship) => (
            <List.Item>
              <Link to={`/system/${ship[0]}`}>{ship[0]}</Link>: {ship[1].length}
              {/* <List
                size="small"
                dataSource={ship[1]}
                renderItem={(ship) => (
                  <List.Item>
                    {ship[0]} {ship[1]} {ship[2]}
                  </List.Item>
                )}
              /> */}
            </List.Item>
          )}
        />
        <List
          header="Trading"
          size="small"
          bordered
          dataSource={Object.entries(requiredShips?.trading.ships ?? {})}
          renderItem={(ship) => (
            <List.Item>
              <Link to={`/system/${ship[0]}`}>{ship[0]}</Link>:
              <List
                size="small"
                dataSource={ship[1]}
                renderItem={(ship) => (
                  <List.Item>
                    {ship[0]} {ship[1]} {ship[2]}
                  </List.Item>
                )}
              />
            </List.Item>
          )}
        />
      </Flex>
    </div>
  );
}
