import { memo } from "react";
import { Handle, Position, type NodeProps } from "@xyflow/react";
import type { OutputFlowNode } from "./flow";
import { NodeFrame } from "./NodeFrame";

export const OutputNode = memo(
  ({
    data,
    selected,
  }: NodeProps<OutputFlowNode>) => (
    <NodeFrame title="Output" selected={selected} minWidth={180} minHeight={140}>
      <Handle
        type="target"
        id="input"
        position={Position.Left}
      />
      <pre className="node__value">
        {data.value === undefined
          ? "Waiting for a run…"
          : JSON.stringify(data.value, null, 2)}
      </pre>
    </NodeFrame>
  ),
);
