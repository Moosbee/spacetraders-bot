import { useQuery } from "@apollo/client/react";
import { Button, Col, Divider, Flex, Result, Row, Space, Spin } from "antd";
import { Link } from "react-router-dom";
import { Fragment } from "react/jsx-runtime";
import PageTitle from "../features/PageTitle";
import RenderSupplyChainNeeds from "../features/RenderSupplyChainNeeds/RenderSupplyChainNeeds";
import { GET_TOTAL_SUPPLY_CHAIN } from "../graphql/queries";

export default function SupplyChain() {
  const { loading, error, data, refetch } = useQuery(GET_TOTAL_SUPPLY_CHAIN);

  if (error) {
    return (
      <Result
        status="error"
        title="TradeSymbol Error"
        subTitle={error.message}
        extra={[
          <Button key="retry" type="primary" onClick={() => refetch()}>
            Try Again
          </Button>,
        ]}
      />
    );
  }

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title={`Supply Chain`} />
      <Space>
        <h1>Supply Chain ({data?.tradeSymbolInfos.length})</h1>
        <Button
          onClick={() => {
            refetch();
          }}
        >
          Reload
        </Button>
      </Space>
      <Divider />
      <Spin spinning={loading}>
        <Flex vertical>
          {data?.tradeSymbolInfos.map((tradeSymbolInfo, index) => (
            <Fragment key={tradeSymbolInfo.symbol}>
              {index !== 0 && <Divider />}
              <Row gutter={16}>
                <Col span={8}>
                  <h2>Needs</h2>
                  <ul>
                    {tradeSymbolInfo.requires.items.map((need) => (
                      <li key={need.symbol}>
                        <Link to={`/supplyChain/${need.symbol}`}>
                          {need.symbol}
                        </Link>
                      </li>
                    ))}
                  </ul>
                </Col>
                <Col span={8}>
                  <div className="flex flex-col justify-center align-center">
                    {RenderSupplyChainNeeds(
                      {
                        symbol: tradeSymbolInfo.symbol,
                        requires: {
                          items: [
                            {
                              symbol: tradeSymbolInfo.symbol,
                              requires: tradeSymbolInfo.requires,
                            },
                          ],
                        },
                      },
                      true,
                    )}
                  </div>
                </Col>
                <Col span={8}>
                  <h2>Needed by</h2>

                  <ul>
                    {tradeSymbolInfo.requiredBy.items.map((need) => (
                      <li key={need.symbol}>
                        {" "}
                        <Link to={`/supplyChain/${need.symbol}`}>
                          {need.symbol}
                        </Link>
                      </li>
                    ))}
                  </ul>
                </Col>
              </Row>
            </Fragment>
          ))}
        </Flex>
      </Spin>
    </div>
  );
}
