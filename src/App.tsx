import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import Workspace from "./Workspace";
import type { FlowGraph, InputFlowNode, PythonFlowNode } from "./flow";

type FlowRunResult = {
  run: {
    status: "success" | "error";
    values: Record<string, unknown>;
    error?: string;
  };
};

function App() {
  const [isRunning, setIsRunning] = useState(false);
  const [runError, setRunError] = useState<string | null>(null);
  const [flow, setFlow] = useState<FlowGraph | null>(null);
  const [input, setInput] = useState("");
  const [code, setCode] = useState("");
  const [output, setOutput] = useState<unknown>(undefined);

  useEffect(() => {
    void invoke<FlowGraph>("get_flow")
      .then((graph) => {
        setFlow(graph);
        const inputNode = graph.nodes.find(
          (node): node is InputFlowNode => node.type === "inputNode",
        );
        const pythonNode = graph.nodes.find(
          (node): node is PythonFlowNode => node.type === "pythonNode",
        );
        setInput(inputNode?.data.value ?? "");
        setCode(pythonNode?.data.code ?? "");
      })
      .catch((error) => setRunError(String(error)));
  }, []);

  async function runFlow() {
    setIsRunning(true);
    setRunError(null);
    setOutput(undefined);
    try {
      const result = await invoke<FlowRunResult>("run_flow", {
        input: JSON.parse(input),
        code,
      });
      if (result.run.status === "error") {
        setRunError(result.run.error ?? "Flow failed.");
      } else {
        setOutput(result.run.values.output);
      }
    } catch (error) {
      setRunError(
        error instanceof SyntaxError
          ? "Input must be valid JSON."
          : String(error),
      );
    } finally {
      setIsRunning(false);
    }
  }

  return (
    <main className="container">
      <div className="floating-panel">
        <h1>cathopper</h1>
        <button type="button" onClick={runFlow} disabled={isRunning || !flow}>
          {isRunning ? "Running…" : "Run flow"}
        </button>
        {runError && <p>{runError}</p>}
      </div>

      {flow && (
        <Workspace
          graph={flow}
          input={input}
          onInputChange={setInput}
          code={code}
          onCodeChange={setCode}
          output={output}
        />
      )}
    </main>
  );
}

export default App;
