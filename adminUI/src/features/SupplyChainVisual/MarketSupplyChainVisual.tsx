import Dagre from "@dagrejs/dagre";
import {
  Edge,
  Panel,
  ReactFlow,
  useEdgesState,
  useNodesState,
  useReactFlow,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { useCallback, useEffect, useMemo } from "react";
import { GetSystemMarketsQuery, MarketTradeGoodType } from "../../gql/graphql";
import { useAppSelector } from "../../redux/hooks";
import { selectDarkMode } from "../../redux/slices/configSlice";
import MarketTradeGoodNode from "./MarketTradeGoodNode";

const nodeTypes = {
  marketTradeGood: MarketTradeGoodNode,
};

const getLayoutedElements = <T,>(
  nodes: {
    id: string;
    position: {
      x: number;
      y: number;
    };
    type: string;
    data: T;
    measured?: { width: number; height: number };
  }[],
  edges: Edge[],
  options: { direction: string },
) => {
  const g = new Dagre.graphlib.Graph().setDefaultEdgeLabel(() => ({}));
  g.setGraph({ rankdir: options.direction });

  edges.forEach((edge) => g.setEdge(edge.source, edge.target));
  nodes.forEach((node) =>
    g.setNode(node.id, {
      ...node,
      width: node.measured?.width ?? 0,
      height: node.measured?.height ?? 0,
    }),
  );

  Dagre.layout(g);

  return {
    nodes: nodes.map((node) => {
      const position = g.node(node.id);
      // We are shifting the dagre node position (anchor=center center) to the top left
      // so it matches the React Flow node anchor point (top left).
      const x = position.x - (node.measured?.width ?? 0) / 2;
      const y = position.y - (node.measured?.height ?? 0) / 2;

      return { ...node, position: { x, y } };
    }),
    edges,
  };
};

function MarketSupplyChainVisual({
  marketTrades,
}: {
  marketTrades: GetSystemMarketsQuery["system"]["marketTrades"]["items"];
}) {
  const isDarkMode = useAppSelector(selectDarkMode);
  const { fitView } = useReactFlow();

  const initialNodes = useMemo(() => {
    return marketTrades.map((t) => ({
      id: t.symbol + t.waypointSymbol,
      position: { x: 0, y: 0 },
      type: "marketTradeGood",
      data: { tradeGood: t },
    }));
  }, [marketTrades]);

  const initalEdges = useMemo(() => {
    return marketTrades
      .map((t1) => {
        return [
          ...marketTrades
            .filter(
              (t2) =>
                t2.waypointSymbol !== t1.waypointSymbol &&
                t2.symbol === t1.symbol &&
                t1.type === MarketTradeGoodType.Export &&
                t2.type === MarketTradeGoodType.Import,
            )
            .map((t2) => ({
              id: t1.symbol + t2.symbol + t1.waypointSymbol + t2.waypointSymbol,
              source: t1.symbol + t1.waypointSymbol,
              target: t2.symbol + t2.waypointSymbol,
              style: {
                strokeWidth: 2,
                stroke: "#E9FF1F",
              },
            })),
          ...marketTrades
            .filter(
              (t2) =>
                t2.waypointSymbol === t1.waypointSymbol &&
                t2.tradeSymbolInfo.requires.items.some(
                  (r) => r.symbol === t1.symbol,
                ) &&
                t1.type === MarketTradeGoodType.Import &&
                t2.type === MarketTradeGoodType.Export,
            )
            .map((t2) => ({
              id: t1.symbol + t2.symbol + t1.waypointSymbol + t2.waypointSymbol,
              source: t1.symbol + t1.waypointSymbol,
              target: t2.symbol + t2.waypointSymbol,
              style: {
                strokeWidth: 2,
                stroke: "#E9FF1F",
              },
            })),
        ];
      })
      .flat();
  }, [marketTrades]);

  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initalEdges);
  useEffect(() => {
    const layouted = getLayoutedElements(initialNodes, initalEdges, {
      direction: "LR",
    });

    setNodes([...layouted.nodes]);
    setEdges([...layouted.edges]);

    fitView();
  }, [fitView, initalEdges, initialNodes, setEdges, setNodes]);

  const onLayout = useCallback(
    (direction: string) => {
      const layouted = getLayoutedElements(nodes, edges, { direction });

      setNodes([...layouted.nodes]);
      setEdges([...layouted.edges]);

      fitView();
    },
    [nodes, edges, setNodes, setEdges, fitView],
  );

  return (
    <div className="w-[calc(100%)]  h-[calc(100vh-10rem)]">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        fitView
        nodeTypes={nodeTypes}
        colorMode={isDarkMode ? "dark" : "light"}
        minZoom={0.1}
      >
        <Panel position="top-right">
          <button onClick={() => onLayout("LR")}>Reset</button>
        </Panel>
      </ReactFlow>
    </div>
  );
}

export default MarketSupplyChainVisual;
