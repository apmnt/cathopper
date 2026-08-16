import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import Workspace from "./Workspace";
import type { FlowGraph } from "./flow";

type FlowRunResult = {
  status: "success" | "error";
  values: Record<string, unknown>;
  error?: string;
};

function App() {
  const [isRunning, setIsRunning] = useState(false);
  const [runError, setRunError] = useState<string | null>(null);
  const [flow, setFlow] = useState<FlowGraph | null>(null);
  const [values, setValues] = useState<Record<string, unknown>>({});

  useEffect(() => {
    void invoke<FlowGraph>("get_flow")
      .then((graph) => {
        setFlow(graph);
      })
      .catch((error) => setRunError(String(error)));
  }, []);

  async function runFlow() {
    if (!flow) return;
    setIsRunning(true);
    setRunError(null);
    setValues({});
    try {
      const result = await invoke<FlowRunResult>("run_flow", {
        flow,
      });
      if (result.status === "error") {
        setRunError(result.error ?? "Flow failed.");
      } else {
        setValues(result.values);
      }
    } catch (error) {
      setRunError(String(error));
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
          onGraphChange={setFlow}
          values={values}
        />
      )}
    </main>
  );
}

export default App;
