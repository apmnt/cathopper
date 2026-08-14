import { memo, useCallback } from "react";
import { Handle, Position, type Node, type NodeProps } from "@xyflow/react";
import Editor from "@monaco-editor/react";
import { NodeFrame } from "./NodeFrame";
import type { editor } from "monaco-editor";

export type EditorNodeData = {
  code: string;
  onCodeChange: (code: string) => void;
};

type EditorNodeType = Node<EditorNodeData>;

const editorOptions: editor.IStandaloneEditorConstructionOptions = {
  minimap: { enabled: false },
  fontSize: 12,
  lineNumbers: "off" as const,
  padding: { top: 8 },
  scrollBeyondLastLine: false,
  renderLineHighlight: "line" as const,
  renderLineHighlightOnlyWhenFocus: true,
};

const EditorNode = memo(
  ({ data, selected }: NodeProps<EditorNodeType>) => {
    const onEditorChange = useCallback(
      (value: string | undefined) => data.onCodeChange(value ?? ""),
      [data.onCodeChange],
    );
    return (
      <NodeFrame
        title="Code"
        selected={selected}
        minWidth={240}
        minHeight={180}
      >
        <Handle
          type="target"
          position={Position.Left}
        />

        <div className="nodrag nowheel node__editor">
          <Editor
            width="100%"
            height="100%"
            defaultLanguage="python"
            theme="vs-light"
            value={data.code}
            onChange={onEditorChange}
            options={editorOptions}
          />
        </div>

        <Handle
          type="source"
          position={Position.Right}
        />
      </NodeFrame>
    );
  },
);

export default EditorNode;
