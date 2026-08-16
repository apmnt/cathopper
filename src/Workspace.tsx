import {
  addEdge,
  applyEdgeChanges,
  applyNodeChanges,
  Background,
  Controls,
  MiniMap,
  ReactFlow,
  type NodeTypes,
  type OnConnect,
} from "@xyflow/react";
import { InputNode } from "./FlowNodes";
import { OutputNode } from "./OutputNode";
import { PythonNode } from "./PythonNode";
import type { FlowGraph, FlowNode } from "./flow";

const nodeTypes = {
  inputNode: InputNode,
  pythonNode: PythonNode,
  outputNode: OutputNode,
} satisfies Record<FlowNode["type"], NodeTypes[string]>;

type NodeDataChange = { value: string } | { code: string };
type UpdateNodeData = (nodeId: string, change: NodeDataChange) => void;

const withNodeData = <Node extends FlowNode>(
  node: Node,
  change: Partial<Node["data"]>,
): Node => ({
  ...node,
  data: { ...node.data, ...change },
});

const decorateNode = (
  updateNodeData: UpdateNodeData,
  values: Record<string, unknown>,
) =>
  (node: FlowNode): FlowNode => {
    switch (node.type) {
      case "inputNode":
        return withNodeData(node, {
          onChange: (value) => updateNodeData(node.id, { value }),
        });
      case "pythonNode":
        return withNodeData(node, {
          onChange: (code) => updateNodeData(node.id, { code }),
        });
      case "outputNode":
        return withNodeData(node, { value: values[node.id] });
    }
  };

const Workspace = ({
  graph,
  onGraphChange,
  values,
}: {
  graph: FlowGraph;
  onGraphChange: (graph: FlowGraph) => void;
  values: Record<string, unknown>;
}) => {
  const updateNodeData = (
    nodeId: string,
    change: NodeDataChange,
  ) =>
    onGraphChange({
      ...graph,
      nodes: graph.nodes.map((node) =>
        node.id === nodeId
          ? ({ ...node, data: { ...node.data, ...change } } as FlowNode)
          : node,
      ),
    });
  const nodes = graph.nodes.map(decorateNode(updateNodeData, values));
  const onConnect: OnConnect = (connection) =>
    onGraphChange({
      ...graph,
      edges: addEdge(connection, graph.edges),
    });

  return (
    <ReactFlow
      className="nodes-canvas"
      nodes={nodes}
      edges={graph.edges}
      onNodesChange={(changes) =>
        onGraphChange({
          ...graph,
          nodes: applyNodeChanges(changes, graph.nodes),
        })
      }
      onEdgesChange={(changes) =>
        onGraphChange({
          ...graph,
          edges: applyEdgeChanges(changes, graph.edges),
        })
      }
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
