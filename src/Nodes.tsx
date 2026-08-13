import { useEffect, useCallback } from "react";
import {
  type Edge,
  type Node,
  type OnConnect,
  type NodeTypes,
  ReactFlow,
  useNodesState,
  useEdgesState,
  addEdge,
  Background,
  MiniMap,
  Controls,
  Position,
} from "@xyflow/react";

import EditorNode from "./EditorNode";
import type { EditorNodeData } from "./EditorNode";

const nodeTypes: NodeTypes = {
  editorNode: EditorNode,
};

const defaultViewport = { x: 0, y: 0, zoom: 1.5 };

type FlowNodeData = { label: string } | EditorNodeData;

type FlowNode = Node<FlowNodeData>;

const Nodes = () => {
  const [nodes, setNodes, onNodesChange] = useNodesState<FlowNode>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);
  useEffect(() => {
    setNodes([
      {
        id: "1",
        type: "input",
        data: { label: "An input node" },
        position: { x: 0, y: 40 },
        sourcePosition: Position.Right,
        style: { width: 160, height: 40 },
      },
      {
        id: "2",
        type: "editorNode",
        data: {
          code: "const message = 'Hello from cathopper';",
          onCodeChange: (code: string) => {
            setNodes((nds) =>
              nds.map((node) =>
                node.id === "2"
                  ? { ...node, data: { ...node.data, code } }
                  : node,
              ),
            );
          },
        },
        position: { x: 200, y: 0 },
        style: { width: 240, height: 180 },
      },
      {
        id: "3",
        type: "output",
        data: { label: "Output A" },
        position: { x: 600, y: 40 },
        targetPosition: Position.Left,
        style: { width: 160, height: 40 },
      },
      {
        id: "4",
        type: "output",
        data: { label: "Output B" },
        position: { x: 600, y: 120 },
        targetPosition: Position.Left,
        style: { width: 160, height: 40 },
      },
    ]);

    setEdges([
      {
        id: "e1-2",
        source: "1",
        target: "2",
      },
      {
        id: "e2a-3",
        source: "2",
        target: "3",
      },
      {
        id: "e2b-4",
        source: "2",
        target: "4",
      },
    ]);
  }, []);

  const onConnect: OnConnect = useCallback(
    (params) => setEdges((eds) => addEdge(params, eds)),
    [],
  );
  return (
    <ReactFlow
      nodes={nodes}
      edges={edges}
      onNodesChange={onNodesChange}
      onEdgesChange={onEdgesChange}
      onConnect={onConnect}
      nodeTypes={nodeTypes}
      snapToGrid={false}
      defaultEdgeOptions={{
        type: "default",
        animated: false,
      }}
      defaultViewport={defaultViewport}
      fitView
      attributionPosition="bottom-left"
      colorMode="system"
    >
      <Background />
      <MiniMap />
      <Controls />
    </ReactFlow>
  );
};

export default Nodes;
