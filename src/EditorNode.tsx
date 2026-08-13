import { memo, useCallback, useMemo, useState } from "react";
import { Handle, Position, type Node, type NodeProps } from "@xyflow/react";
import Editor from "@monaco-editor/react";
import "./EditorNode.css";

export type EditorNodeData = {
  code: string;
  onCodeChange: (code: string) => void;
};

type EditorNodeType = Node<EditorNodeData>;

const EditorNode = memo(
  ({ data, isConnectable }: NodeProps<EditorNodeType>) => {
    const [overflowWidgetsDomNode, setOverflowWidgetsDomNode] =
      useState<HTMLDivElement | null>(null);
    const onEditorChange = useCallback(
      (value: string | undefined) => data.onCodeChange(value ?? ""),
      [data.onCodeChange],
    );
    const editorOptions = useMemo(
      () => ({
        overflowWidgetsDomNode: overflowWidgetsDomNode ?? undefined,
        fixedOverflowWidgets: false,
        minimap: { enabled: false },
        fontSize: 12,
        lineNumbers: "off" as const,
        padding: { top: 8 },
        scrollBeyondLastLine: false,
        renderLineHighlight: "line" as const,
        renderLineHighlightOnlyWhenFocus: true,
      }),
      [overflowWidgetsDomNode],
    );

    return (
      <div className="editor-node">
        <Handle
          type="target"
          position={Position.Left}
          isConnectable={isConnectable}
        />

        <div className="editor-node__header">Code</div>

        <div className="nodrag nowheel editor-node__editor">
          {overflowWidgetsDomNode && (
            <Editor
              width="100%"
              height="100%"
              defaultLanguage="typescript"
              theme="vs-light"
              value={data.code}
              onChange={onEditorChange}
              options={editorOptions}
            />
          )}
        </div>

        <div
          ref={setOverflowWidgetsDomNode}
          className="monaco-editor vs nodrag nowheel editor-node__widgets"
        />

        <Handle
          type="source"
          position={Position.Right}
          isConnectable={isConnectable}
        />
      </div>
    );
  },
);

export default EditorNode;
