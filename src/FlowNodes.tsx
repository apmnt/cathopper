import { memo } from "react";
import { Handle, Position, type NodeProps } from "@xyflow/react";
import type { InputFlowNode } from "./flow";
import { NodeFrame } from "./NodeFrame";

export const InputNode = memo(
  ({
    data,
    selected,
  }: NodeProps<InputFlowNode>) => (
    <NodeFrame title="Input" selected={selected} minWidth={180} minHeight={130}>
      <textarea
        className="nodrag nowheel node__textarea"
        value={data.value}
        onChange={(event) => data.onChange?.(event.currentTarget.value)}
      />
      <Handle
        type="source"
        id="output"
        position={Position.Right}
      />
    </NodeFrame>
  ),
);
