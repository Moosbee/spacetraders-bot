import { useQuery } from "@apollo/client/react";
import {
  Button,
  Card,
  Descriptions,
  Divider,
  Flex,
  List,
  Space,
  Spin,
  Table,
  TableProps,
} from "antd";
import { Link, useParams } from "react-router-dom";
import PageTitle from "../features/PageTitle";
import WaypointLink from "../features/WaypointLink";
import {
  GetSystemQuery,
  WaypointModifierSymbol,
  WaypointTraitSymbol,
  WaypointType,
} from "../gql/graphql";
import { GET_SYSTEM } from "../graphql/queries";

function System() {
  const { systemID } = useParams();

  const { loading, error, data, dataState, refetch } = useQuery(GET_SYSTEM, {
    variables: { systemSymbol: systemID || "" },
  });

  // if (dataState != "complete") return <p>Loading...</p>;
  if (error) return <p>Error: {error.message}</p>;

  const system = data?.system;

  type GQLWaypoint = GetSystemQuery["system"]["waypoints"][number];

  const items = [
    {
      label: "Symbol",
      key: "symbol",
      children: system?.symbol,
    },
    {
      label: "Sector Symbol",
      key: "sectorSymbol",
      children: system?.sectorSymbol,
    },
    {
      key: "reload",
      children: (
        <>
          <Button
            onClick={() => {
              refetch();
            }}
          >
            Reload
          </Button>
          <Spin spinning={loading || dataState !== "complete"} />
        </>
      ),
    },
    {
      label: "System Type",
      key: "systemType",
      children: system?.systemType,
    },
    {
      label: "Fleets",
      key: "Fleets",
      children: system?.fleets.length,
    },
    {
      key: "Map",
      children: <Link to={`/map/system/${systemID}`}>Map</Link>,
    },
    {
      label: "X Coordinate",
      key: "x",
      children: system?.x,
    },
    {
      label: "Y Coordinate",
      key: "y",
      children: system?.y,
    },

    {
      key: "request",
      children: (
        <>
          <Button
            onClick={() => {
              alert("Todo");
            }}
          >
            Request
          </Button>
          <Spin spinning={loading || dataState !== "complete"} />
        </>
      ),
    },
    {
      label: "Waypoints",
      key: "waypoints",
      children: `${system?.waypoints.filter((wp) => wp.chartedOn).length}/${
        system?.waypoints.length
      }`,
    },
    {
      label: "Marketplaces",
      key: "marketplaces",
      children: `${
        system?.waypoints
          .filter((wp) => wp.hasMarketplace)
          .filter((wp) => wp.chartedOn).length
      }/${system?.waypoints.filter((wp) => wp.hasMarketplace).length}`,
    },
    {
      label: "Shipyards",
      key: "shipyards",
      children: `${
        system?.waypoints
          .filter((wp) => wp.hasShipyard)
          .filter((wp) => wp.chartedOn).length
      }/${system?.waypoints.filter((wp) => wp.hasShipyard).length}`,
    },
  ];

  const columns: TableProps<GQLWaypoint>["columns"] = [
    {
      title: "Symbol",
      dataIndex: "symbol",
      key: "symbol",
      render: (symbol: string) => (
        <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
      ),
      sorter: (a, b) => a.symbol.localeCompare(b.symbol),
    },
    {
      title: "Type",
      dataIndex: "waypointType",
      key: "waypointType",
      sorter: (a, b) => a.waypointType.localeCompare(b.waypointType),
      filters: Object.values(WaypointType).map((type) => ({
        text: type,
        value: type,
      })),
      onFilter: (value, record) => record.waypointType === value,
    },
    {
      title: "Pos X",
      dataIndex: "x",
      key: "x",
      sorter: (a, b) => a.x - b.x,
    },
    {
      title: "Pos Y",
      dataIndex: "y",
      key: "y",
      sorter: (a, b) => a.y - b.y,
    },
    {
      title: "Orbitals",
      dataIndex: "orbitals",
      key: "orbitals",
      render: (orbitals: string[]) =>
        orbitals.length > 0 ? (
          <Flex gap={1} vertical>
            {orbitals.map((symbol) => (
              <WaypointLink waypoint={symbol} key={symbol}>
                {symbol.replace(system?.symbol + "-", "")}
              </WaypointLink>
            ))}
          </Flex>
        ) : (
          "None"
        ), // List symbols of orbitals or "None"
      sorter: (a, b) => a.orbitals.length - b.orbitals.length,
    },
    {
      title: "Orbits",
      dataIndex: "orbits",
      key: "orbits",
      render: (orbits: string) =>
        orbits ? (
          <WaypointLink waypoint={orbits}>
            {orbits.replace(system?.symbol + "-", "")}
          </WaypointLink>
        ) : (
          "N/A"
        ), // Display "N/A" if undefined
      sorter: (a, b) => (a.orbits ?? "").localeCompare(b.orbits ?? ""),
    },
    {
      title: "Faction",
      dataIndex: "faction",
      key: "faction",
      render: (faction) => (faction ? faction : "N/A"), // Display faction symbol or "N/A"
      sorter: (a, b) => (a.faction ?? "").localeCompare(b.faction ?? ""),
    },
    {
      title: "Traits",
      dataIndex: "traits",
      key: "traits",
      render: (traits) => (
        <Flex gap={1} vertical>
          {traits.map((trait: WaypointTraitSymbol) => (
            <span key={trait}>{trait}</span>
          ))}
        </Flex>
      ), // List names of traits
      sorter: (a, b) => a.traits.length - b.traits.length,
      filters: Object.values(WaypointTraitSymbol).map((trait) => ({
        text: trait,
        value: trait,
      })),
      onFilter: (value, record) => record.traits.some((t) => t === value),
    },
    {
      title: "Modifiers",
      dataIndex: "modifiers",
      key: "modifiers",
      render: (modifiers) =>
        modifiers && modifiers.length > 0 ? (
          <span>
            {modifiers?.map((modifier: WaypointModifierSymbol) => (
              <span key={modifier}>{modifier}</span>
            ))}
          </span>
        ) : (
          "None"
        ),
      sorter: (a, b) => (a.modifiers?.length ?? 0) - (b.modifiers?.length ?? 0),
      filters: Object.values(WaypointModifierSymbol).map((modifier) => ({
        text: modifier,
        value: modifier,
      })),
      onFilter: (value, record) =>
        record.modifiers?.some((m) => m === value) ?? false,
    },
    // {
    //   title: "Trade Goods",
    //   dataIndex: "trade_goods",
    //   key: "trade_goods",
    //   render: (
    //     trade_goods:
    //       | {
    //           symbol: TradeSymbol;
    //           type: MarketTradeGoodTypeEnum;
    //         }[]
    //       | undefined
    //   ) =>
    //     trade_goods && trade_goods.length > 0 ? (
    //       <>
    //         {/* <Flex gap={1} vertical>
    //           {trade_goods.map((trade_good) => (
    //             <span>
    //               {trade_good.type.slice(0, 3)} {trade_good.symbol}
    //             </span>
    //           ))}
    //         </Flex> */}
    //         <Popover
    //           content={
    //             <Flex gap={1} vertical>
    //               {trade_goods.map((trade_good) => (
    //                 <span key={trade_good.symbol}>
    //                   {trade_good.type.slice(0, 3)} {trade_good.symbol}
    //                 </span>
    //               ))}
    //             </Flex>
    //           }
    //         >
    //           <Flex gap={1} vertical>
    //             {trade_goods.filter((t) => t.type === "EXCHANGE").length >
    //               0 && (
    //               <span>
    //                 EXCHANGE{" "}
    //                 {trade_goods.filter((t) => t.type === "EXCHANGE").length}
    //               </span>
    //             )}
    //             {trade_goods.filter((t) => t.type === "IMPORT").length > 0 && (
    //               <span>
    //                 IMPORT{" "}
    //                 {trade_goods.filter((t) => t.type === "IMPORT").length}
    //               </span>
    //             )}
    //             {trade_goods.filter((t) => t.type === "EXPORT").length > 0 && (
    //               <span>
    //                 EXPORT{" "}
    //                 {trade_goods.filter((t) => t.type === "EXPORT").length}
    //               </span>
    //             )}
    //           </Flex>
    //         </Popover>
    //       </>
    //     ) : (
    //       "None"
    //     ),
    //   sorter: (a, b) =>
    //     (a.trade_goods?.length ?? 0) - (b.trade_goods?.length ?? 0),
    //   filters: Object.values(TradeSymbol).map((trade_good) => ({
    //     text: trade_good,
    //     value: trade_good,
    //   })),
    //   onFilter: (value, record) =>
    //     record.trade_goods?.some((t) => t.symbol === value) ?? false,
    // },
    {
      title: "Chart by",
      dataIndex: "chartedBy",
      key: "chartedBy",
      render: (chartedBy) => (chartedBy ? chartedBy : "N/A"), // Display chart symbol or "N/A"
      sorter: (a, b) => (a.chartedBy ?? "").localeCompare(b.chartedBy ?? ""),
    },
    {
      title: "Chart on",
      dataIndex: "chartedOn",
      key: "chartedOn",
      render: (chartedOn) =>
        chartedOn ? new Date(chartedOn).toLocaleString() : "N/A", // Display chart symbol or "N/A"
      sorter: (a, b) => (a.chartedOn ?? "").localeCompare(b.chartedOn ?? ""),
    },
    {
      title: "Construction",
      dataIndex: "isUnderConstruction",
      key: "isUnderConstruction",
      render: (value) => (value ? "Yes" : "No"), // Render boolean as "Yes" or "No"
      sorter: (a, b) =>
        (a.isUnderConstruction ? 1 : 0) - (b.isUnderConstruction ? 1 : 0),
      filters: [
        { text: "Yes", value: true },
        { text: "No", value: false },
      ],
      onFilter: (value, record) => record.isUnderConstruction === value,
    },
    {
      title: "Has Shipyard",
      dataIndex: "hasShipyard",
      key: "hasShipyard",
      render: (value) => (value ? "Yes" : "No"), // Render boolean as "Yes" or "No"
      sorter: (a, b) => (a.hasShipyard ? 1 : 0) - (b.hasShipyard ? 1 : 0),
      filters: [
        { text: "Yes", value: true },
        { text: "No", value: false },
      ],
      onFilter: (value, record) => record.hasShipyard === value,
    },
    {
      title: "Has Market",
      dataIndex: "hasMarketplace",
      key: "hasMarketplace",
      render: (value) => (value ? "Yes" : "No"), // Render boolean as "Yes" or "No"
      sorter: (a, b) => (a.hasMarketplace ? 1 : 0) - (b.hasMarketplace ? 1 : 0),
      filters: [
        { text: "Yes", value: true },
        { text: "No", value: false },
      ],
      onFilter: (value, record) => record.hasMarketplace === value,
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title={`System ${systemID}`} />
      <h2>System {systemID}</h2>
      <Space>
        <Descriptions bordered column={3} items={items} />
        <Card size="small" title="Known Agents">
          <List
            size="small"
            dataSource={[...(system?.seenAgents || [])].sort(
              (a, b) => b.count - a.count
            )}
            renderItem={(agent) => (
              <List.Item>
                <Link to={`/agents/${agent.symbol}`}>
                  {agent.symbol} ({agent.count})
                </Link>
              </List.Item>
            )}
          />
        </Card>
        <Card size="small" title="Ships in System">
          <List
            size="small"
            style={{ maxHeight: "200px", overflowY: "auto" }}
            dataSource={system?.ships}
            renderItem={(ship) => (
              <List.Item>
                <Link to={`/ships/${ship.symbol}`}>
                  {ship.symbol} ({ship.status.status.__typename}) (
                  {ship.status.tempAssignmentId || ship.status.assignmentId}) (
                  {ship.status.tempFleetId || ship.status.fleetId})
                </Link>
              </List.Item>
            )}
          />
        </Card>
      </Space>
      <Divider />
      <Table
        size="small"
        columns={columns}
        dataSource={system?.waypoints || []}
        rowKey={(row) => row.symbol}
        pagination={{
          showTotal: (total, range) => `${range[0]}-${range[1]} of ${total}`,
        }}
      />
    </div>
  );
}

export default System;
