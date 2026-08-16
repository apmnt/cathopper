import { memo } from "react";
import Editor from "@monaco-editor/react";
import { Handle, Position, type NodeProps } from "@xyflow/react";
import type { PythonFlowNode, Terminal } from "./flow";
import { NodeFrame } from "./NodeFrame";

type TerminalDirection = Terminal["direction"];

const handleProps = {
  input: { type: "target", position: Position.Left },
  output: { type: "source", position: Position.Right },
} as const;

const TerminalHandles = ({
  terminals,
  direction,
}: {
  terminals: Terminal[];
  direction: TerminalDirection;
}) => (
    <div className={`node__terminals node__terminals--${direction}`}>
      {terminals
        .filter((terminal) => terminal.direction === direction)
        .map((terminal) => (
          <div
            className={`node__terminal node__terminal--${direction}`}
            key={terminal.id}
          >
            <Handle {...handleProps[direction]} id={terminal.id} />
            <span className="node__terminal-name">{terminal.name}</span>
          </div>
        ))}
    </div>
);

export const PythonNode = memo(
  ({
    data,
    selected,
  }: NodeProps<PythonFlowNode>) => (
    <NodeFrame title="Python" selected={selected} minWidth={260} minHeight={180}>
      <TerminalHandles terminals={data.terminals} direction="input" />
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
      <TerminalHandles terminals={data.terminals} direction="output" />
    </NodeFrame>
  ),
);
