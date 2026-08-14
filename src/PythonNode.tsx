import { memo } from "react";
import Editor from "@monaco-editor/react";
import { Handle, Position, type NodeProps } from "@xyflow/react";
import type { PythonFlowNode } from "./flow";
import { NodeFrame } from "./NodeFrame";

export const PythonNode = memo(
  ({
    data,
    selected,
  }: NodeProps<PythonFlowNode>) => (
    <NodeFrame title="Python" selected={selected} minWidth={260} minHeight={180}>
      <Handle
        type="target"
        id="input"
        position={Position.Left}
      />
      <div className="nodrag nowheel node__editor">
        <Editor
          width="100%"
          height="100%"
          defaultLanguage="python"
          theme="vs-light"
          value={data.code}
          onChange={(code) => data.onChange?.(code ?? "")}
          options={{
            minimap: { enabled: false },
            fontSize: 12,
            lineNumbers: "off",
            padding: { top: 8 },
            scrollBeyondLastLine: false,
          }}
        />
      </div>
      <Handle
        type="source"
        id="output"
        position={Position.Right}
      />
    </NodeFrame>
  ),
);
