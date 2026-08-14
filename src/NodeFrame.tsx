import { NodeResizer } from "@xyflow/react";
import type { ReactNode } from "react";
import "./NodeFrame.css";

export function NodeFrame({
  title,
  selected,
  children,
  minWidth,
  minHeight,
}: {
  title: string;
  selected: boolean;
  children: ReactNode;
  minWidth: number;
  minHeight: number;
}) {
  return (
    <div className="node">
      <NodeResizer
        isVisible={selected}
        minWidth={minWidth}
        minHeight={minHeight}
        color="#396cd8"
      />
      <header className="node__header">{title}</header>
      {children}
    </div>
  );
}
