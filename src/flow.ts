import { type Edge, type Node } from "@xyflow/react";

export type InputNodeData = {
  value: string;
  onChange?: (value: string) => void;
};
export type PythonNodeData = {
  code: string;
  onChange?: (code: string) => void;
};
export type OutputNodeData = { value?: unknown };

export type InputFlowNode = Node<InputNodeData, "inputNode">;
export type PythonFlowNode = Node<PythonNodeData, "pythonNode">;
export type OutputFlowNode = Node<OutputNodeData, "outputNode">;

export type FlowNode =
  | InputFlowNode
  | PythonFlowNode
  | OutputFlowNode;

export type FlowEdge = Edge;

export type FlowGraph = {
  nodes: FlowNode[];
  edges: FlowEdge[];
};
