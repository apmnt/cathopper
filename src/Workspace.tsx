import {
  addEdge,
  Background,
  Controls,
  MiniMap,
  ReactFlow,
  type NodeTypes,
  type OnConnect,
  useEdgesState,
  useNodesState,
} from "@xyflow/react";
import { useCallback, useEffect } from "react";
import { InputNode } from "./FlowNodes";
import { OutputNode } from "./OutputNode";
import { PythonNode } from "./PythonNode";
import type { FlowEdge, FlowGraph, FlowNode } from "./flow";

const nodeTypes = {
  inputNode: InputNode,
  pythonNode: PythonNode,
  outputNode: OutputNode,
} satisfies Record<FlowNode["type"], NodeTypes[string]>;

const Workspace = ({
  graph,
  input,
  onInputChange,
  code,
  onCodeChange,
  output,
}: {
  graph: FlowGraph;
  input: string;
  onInputChange: (value: string) => void;
  code: string;
  onCodeChange: (code: string) => void;
  output: unknown;
}) => {
  const [edges, setEdges, onEdgesChange] = useEdgesState<FlowEdge>([]);
  const [nodes, setNodes, onNodesChange] = useNodesState<FlowNode>([]);
  const onConnect: OnConnect = useCallback(
    (connection) => setEdges((current) => addEdge(connection, current)),
    [setEdges],
  );

  useEffect(() => {
    setNodes(graph.nodes);
    setEdges(graph.edges);
  }, [graph, setEdges, setNodes]);

  useEffect(() => {
    setNodes((current) =>
      current.map((node) =>
        node.type === "inputNode"
          ? { ...node, data: { value: input, onChange: onInputChange } }
          : node.type === "pythonNode"
            ? { ...node, data: { code, onChange: onCodeChange } }
            : node.type === "outputNode"
              ? { ...node, data: { value: output } }
              : node,
      ) as FlowNode[],
    );
  }, [code, input, onCodeChange, onInputChange, output, setNodes]);

  return (
    <ReactFlow
      className="nodes-canvas"
      nodes={nodes}
      edges={edges}
      onNodesChange={onNodesChange}
      onEdgesChange={onEdgesChange}
      onConnect={onConnect}
      nodeTypes={nodeTypes}
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

export default Workspace;
