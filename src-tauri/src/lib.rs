use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, VecDeque},
    format, fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::Mutex,
};

const PYTHON_RUNNER: &str = include_str!("../python/cathopper_runner.py");

#[derive(Deserialize)]
struct RunnerError {
    kind: String,
    message: String,
    traceback: Option<String>,
}

#[derive(Deserialize)]
struct RunnerResponse {
    ok: bool,
    output: Option<Value>,
    stdout: String,
    stderr: String,
    error: Option<RunnerError>,
}

#[derive(Serialize)]
struct PythonResult {
    output: Value,
    stdout: String,
    stderr: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct FlowPosition {
    x: f64,
    y: f64,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum FlowNodeKind {
    InputNode,
    PythonNode,
    OutputNode,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct FlowNode {
    id: String,
    #[serde(rename = "type")]
    kind: FlowNodeKind,
    position: FlowPosition,
    data: Value,
    style: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<f64>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct FlowEdge {
    id: String,
    source: String,
    target: String,
    source_handle: Option<String>,
    target_handle: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
struct FlowGraph {
    nodes: Vec<FlowNode>,
    edges: Vec<FlowEdge>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
enum FlowRunStatus {
    Idle,
    Running,
    Success,
    Error,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FlowRunState {
    status: FlowRunStatus,
    values: BTreeMap<String, Value>,
    error: Option<String>,
    failed_node_id: Option<String>,
}

#[derive(Clone, Serialize)]
struct FlowState {
    nodes: Vec<FlowNode>,
    edges: Vec<FlowEdge>,
    run: FlowRunState,
}

struct FlowStore(Mutex<FlowState>);

impl Default for FlowStore {
    fn default() -> Self {
        Self(Mutex::new(FlowState {
            nodes: vec![
                FlowNode {
                    id: "input".into(),
                    kind: FlowNodeKind::InputNode,
                    position: FlowPosition { x: 0.0, y: 120.0 },
                    data: json!({ "value": "21" }),
                    style: json!({ "width": 200, "height": 140 }),
                    width: None,
                    height: None,
                },
                FlowNode {
                    id: "python".into(),
                    kind: FlowNodeKind::PythonNode,
                    position: FlowPosition { x: 270.0, y: 70.0 },
                    data: json!({ "code": "output = input * 2" }),
                    style: json!({ "width": 320, "height": 240 }),
                    width: None,
                    height: None,
                },
                FlowNode {
                    id: "output".into(),
                    kind: FlowNodeKind::OutputNode,
                    position: FlowPosition { x: 660.0, y: 120.0 },
                    data: json!({}),
                    style: json!({ "width": 200, "height": 140 }),
                    width: None,
                    height: None,
                },
            ],
            edges: vec![
                FlowEdge {
                    id: "input-python".into(),
                    source: "input".into(),
                    target: "python".into(),
                    source_handle: Some("output".into()),
                    target_handle: Some("input".into()),
                },
                FlowEdge {
                    id: "python-output".into(),
                    source: "python".into(),
                    target: "output".into(),
                    source_handle: Some("output".into()),
                    target_handle: Some("input".into()),
                },
            ],
            run: FlowRunState {
                status: FlowRunStatus::Idle,
                values: BTreeMap::new(),
                error: None,
                failed_node_id: None,
            },
        }))
    }
}

fn idle_run_state() -> FlowRunState {
    run_state(FlowRunStatus::Idle, BTreeMap::new(), None)
}

fn run_state(
    status: FlowRunStatus,
    values: BTreeMap<String, Value>,
    failure: Option<(String, String)>,
) -> FlowRunState {
    let (failed_node_id, error) = failure
        .map(|(node_id, error)| (Some(node_id), Some(error)))
        .unwrap_or((None, None));

    FlowRunState {
        status,
        values,
        error,
        failed_node_id,
    }
}

fn flow_store_state(store: &tauri::State<FlowStore>) -> Result<FlowState, String> {
    store
        .0
        .lock()
        .map_err(|_| "Flow state is unavailable".to_string())
        .map(|state| state.clone())
}

fn uv_command_failure(action: &str, output: Output) -> Result<(), String> {
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    let detail = if stderr.is_empty() {
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    } else {
        stderr
    };
    Err(format!(
        "uv {action} failed{}",
        if detail.is_empty() {
            String::new()
        } else {
            format!(": {detail}")
        }
    ))
}

fn run_uv(command: &mut Command, action: &str) -> Result<(), String> {
    command
        .output()
        .map_err(|error| format!("Could not start uv for {action}: {error}"))
        .and_then(|output| uv_command_failure(action, output))
}

fn project_path(project_dir: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(project_dir);
    if !path.is_absolute() {
        return Err("Python project directory must be an absolute path".into());
    }
    if !path.is_dir() {
        return Err(format!(
            "Python project directory does not exist: {}",
            path.display()
        ));
    }
    Ok(path)
}

fn default_project_path() -> Result<PathBuf, String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("Could not determine Cathopper project directory")?
        .join("python-flow");
    fs::create_dir_all(&path)
        .map_err(|error| format!("Could not create Python project directory: {error}"))?;
    Ok(path)
}

fn ensure_python_project(project_dir: &Path) -> Result<(), String> {
    let pyproject = project_dir.join("pyproject.toml");
    let lockfile = project_dir.join("uv.lock");

    if !pyproject.exists() {
        let mut init = Command::new("uv");
        init.args([
            "init",
            "--bare",
            "--no-workspace",
            "--vcs",
            "none",
            "--managed-python",
            "--python",
            "3.13",
        ])
        .arg(project_dir);
        run_uv(&mut init, "init")?;
    }

    if !lockfile.exists() {
        let mut lock = Command::new("uv");
        lock.args(["lock", "--project"]).arg(project_dir);
        run_uv(&mut lock, "lock")?;
    }

    Ok(())
}

fn write_python_node(project_dir: &Path, code: &str) -> Result<PathBuf, String> {
    let nodes_dir = project_dir.join("nodes");
    fs::create_dir_all(&nodes_dir)
        .map_err(|error| format!("Could not create Python node directory: {error}"))?;
    let source = format!(
        "def run(input):\n{}\n    return (output,)\n",
        code.lines()
            .map(|line| format!("    {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let path = nodes_dir.join("python_node.py");
    fs::write(&path, source)
        .map_err(|error| format!("Could not write Python node source: {error}"))?;
    Ok(path)
}

fn parse_runner_response(stdout: &[u8]) -> Result<RunnerResponse, String> {
    serde_json::from_slice(stdout)
        .map_err(|error| format!("Python runner returned an invalid response: {error}"))
}

/// Runs a single-input, single-output Python node in its project's uv environment.
#[tauri::command]
fn run_python(
    project_dir: Option<String>,
    code: String,
    input: Value,
) -> Result<PythonResult, String> {
    let project_dir = match project_dir {
        Some(project_dir) => project_path(&project_dir)?,
        None => default_project_path()?,
    };
    ensure_python_project(&project_dir)?;
    let source_path = write_python_node(&project_dir, &code)?;

    let request = json!({ "input": input, "sourcePath": source_path });
    let mut command = Command::new("uv");
    command
        .args(["run", "--project"])
        .arg(&project_dir)
        .args(["--locked", "python", "-c", PYTHON_RUNNER])
        .current_dir(&project_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|error| format!("Could not start uv Python runner: {error}"))?;
    child
        .stdin
        .as_mut()
        .ok_or("Could not open Python runner stdin")?
        .write_all(
            serde_json::to_string(&request)
                .map_err(|error| format!("Could not encode Python request: {error}"))?
                .as_bytes(),
        )
        .map_err(|error| format!("Could not write Python request: {error}"))?;

    let process_output = child
        .wait_with_output()
        .map_err(|error| format!("Could not wait for Python runner: {error}"))?;
    if !process_output.status.success() {
        let stderr = String::from_utf8_lossy(&process_output.stderr)
            .trim()
            .to_owned();
        return Err(if stderr.is_empty() {
            "uv Python runner exited unsuccessfully".into()
        } else {
            format!("uv Python runner failed: {stderr}")
        });
    }

    let response = parse_runner_response(&process_output.stdout)?;
    if response.ok {
        return Ok(PythonResult {
            output: response
                .output
                .ok_or("Python runner succeeded without an output value")?,
            stdout: response.stdout,
            stderr: response.stderr,
        });
    }

    let error = response
        .error
        .ok_or("Python runner failed without an error")?;
    let traceback = error
        .traceback
        .filter(|traceback| !traceback.is_empty())
        .map(|traceback| format!("\n{traceback}"))
        .unwrap_or_default();
    Err(format!(
        "Python {} error: {}{}",
        error.kind, error.message, traceback
    ))
}

#[tauri::command]
fn get_flow(store: tauri::State<FlowStore>) -> Result<FlowState, String> {
    flow_store_state(&store)
}

#[tauri::command]
fn replace_flow(flow: FlowGraph, store: tauri::State<FlowStore>) -> Result<FlowState, String> {
    let mut state = store
        .0
        .lock()
        .map_err(|_| "Flow state is unavailable".to_string())?;
    state.nodes = flow.nodes;
    state.edges = flow.edges;
    state.run = idle_run_state();
    Ok(state.clone())
}

fn input_value(node: &FlowNode) -> Result<Value, String> {
    let value = node
        .data
        .get("value")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Input node \"{}\" needs a JSON value.", node.id))?;
    serde_json::from_str(value)
        .map_err(|error| format!("Input node \"{}\" has invalid JSON: {error}", node.id))
}

fn python_source(node: &FlowNode) -> Result<String, String> {
    node.data
        .get("code")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("Python node \"{}\" needs source code.", node.id))
}

#[derive(Debug)]
struct FlowFailure {
    node_id: String,
    error: String,
    values: BTreeMap<String, Value>,
}

struct ExecutionStep {
    node_id: String,
    input_node_id: Option<String>,
}

struct ExecutionPlan {
    nodes_by_id: BTreeMap<String, FlowNode>,
    steps: Vec<ExecutionStep>,
}

struct GraphIndex {
    incoming: BTreeMap<String, String>,
    outgoing: BTreeMap<String, Vec<String>>,
    indegree: BTreeMap<String, usize>,
}

impl GraphIndex {
    fn new(node_ids: impl Iterator<Item = String>) -> Self {
        Self {
            incoming: BTreeMap::new(),
            outgoing: BTreeMap::new(),
            indegree: node_ids.map(|id| (id, 0)).collect(),
        }
    }
}

fn index_edge(
    mut index: GraphIndex,
    edge: FlowEdge,
    nodes: &BTreeMap<String, FlowNode>,
) -> Result<GraphIndex, FlowFailure> {
    if !nodes.contains_key(&edge.source) || !nodes.contains_key(&edge.target) {
        return Err(FlowFailure {
            node_id: edge.target,
            error: format!("Edge \"{}\" references a missing node.", edge.id),
            values: BTreeMap::new(),
        });
    }
    if index
        .incoming
        .insert(edge.target.clone(), edge.source.clone())
        .is_some()
    {
        return Err(FlowFailure {
            node_id: edge.target,
            error: "Multiple inputs are not supported yet.".into(),
            values: BTreeMap::new(),
        });
    }

    *index
        .indegree
        .get_mut(&edge.target)
        .expect("target was validated") += 1;
    index
        .outgoing
        .entry(edge.source)
        .or_default()
        .push(edge.target);
    Ok(index)
}

fn evaluate_node<F>(
    node: &FlowNode,
    previous_value: Option<Value>,
    input_override: Option<&Value>,
    code_override: Option<&str>,
    execute_python: &mut F,
) -> Result<Value, String>
where
    F: FnMut(String, Value) -> Result<Value, String>,
{
    match node.kind {
        FlowNodeKind::InputNode => input_override
            .cloned()
            .map(Ok)
            .unwrap_or_else(|| input_value(node)),
        FlowNodeKind::PythonNode => previous_value
            .ok_or_else(|| "Python node needs an earlier Input node.".to_string())
            .and_then(|input| {
                code_override
                    .map(str::to_owned)
                    .map(Ok)
                    .unwrap_or_else(|| python_source(node))
                    .and_then(|source| execute_python(source, input))
            }),
        FlowNodeKind::OutputNode => {
            previous_value.ok_or_else(|| "This node needs an earlier Input node.".to_string())
        }
    }
}

fn execution_plan(
    nodes: Vec<FlowNode>,
    edges: Vec<FlowEdge>,
) -> Result<ExecutionPlan, FlowFailure> {
    let nodes_by_id = nodes
        .into_iter()
        .map(|node| (node.id.clone(), node))
        .collect::<BTreeMap<_, _>>();
    let GraphIndex {
        incoming,
        outgoing,
        mut indegree,
    } = edges.into_iter().try_fold(
        GraphIndex::new(nodes_by_id.keys().cloned()),
        |index, edge| index_edge(index, edge, &nodes_by_id),
    )?;

    let mut ready = indegree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(id, _)| id.clone())
        .collect::<VecDeque<_>>();
    let ordered_ids = std::iter::from_fn(|| {
        ready.pop_front().map(|node_id| {
            outgoing
                .get(&node_id)
                .into_iter()
                .flatten()
                .for_each(|target| {
                    let degree = indegree.get_mut(target).expect("target was validated");
                    *degree -= 1;
                    if *degree == 0 {
                        ready.push_back(target.clone());
                    }
                });
            node_id
        })
    })
    .collect::<Vec<_>>();

    if ordered_ids.len() != indegree.len() {
        return Err(FlowFailure {
            node_id: indegree
                .iter()
                .find(|(_, degree)| **degree > 0)
                .map(|(id, _)| id.clone())
                .unwrap_or_default(),
            error: "Flow contains a cycle.".into(),
            values: BTreeMap::new(),
        });
    }

    let steps = ordered_ids
        .into_iter()
        .map(|node_id| ExecutionStep {
            input_node_id: incoming.get(&node_id).cloned(),
            node_id,
        })
        .collect();

    Ok(ExecutionPlan { nodes_by_id, steps })
}

fn evaluate_flow<F>(
    nodes: Vec<FlowNode>,
    edges: Vec<FlowEdge>,
    input_override: Option<&Value>,
    code_override: Option<&str>,
    mut execute_python: F,
) -> Result<BTreeMap<String, Value>, FlowFailure>
where
    F: FnMut(String, Value) -> Result<Value, String>,
{
    let ExecutionPlan { nodes_by_id, steps } = execution_plan(nodes, edges)?;

    steps
        .into_iter()
        .try_fold(BTreeMap::new(), |mut values, step| {
            let previous_value = step
                .input_node_id
                .as_ref()
                .and_then(|source_id| values.get(source_id))
                .cloned();
            let value = evaluate_node(
                nodes_by_id.get(&step.node_id).expect("planned node exists"),
                previous_value,
                input_override,
                code_override,
                &mut execute_python,
            )
            .map_err(|error| FlowFailure {
                node_id: step.node_id.clone(),
                error,
                values: values.clone(),
            })?;

            values.insert(step.node_id, value);
            Ok(values)
        })
}

#[tauri::command]
fn run_flow(
    store: tauri::State<FlowStore>,
    input: Option<Value>,
    code: Option<String>,
) -> Result<FlowState, String> {
    let (nodes, edges) = {
        let mut state = store
            .0
            .lock()
            .map_err(|_| "Flow state is unavailable".to_string())?;
        state.run = run_state(FlowRunStatus::Running, BTreeMap::new(), None);
        (state.nodes.clone(), state.edges.clone())
    };

    let run = evaluate_flow(
        nodes,
        edges,
        input.as_ref(),
        code.as_deref(),
        |source, input| run_python(None, source, input).map(|result| result.output),
    );

    let mut state = store
        .0
        .lock()
        .map_err(|_| "Flow state is unavailable".to_string())?;
    state.run = match run {
        Ok(values) => run_state(FlowRunStatus::Success, values, None),
        Err(FlowFailure {
            node_id,
            error,
            values,
        }) => run_state(FlowRunStatus::Error, values, Some((node_id, error))),
    };
    Ok(state.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flow_node(id: &str, kind: FlowNodeKind, data: Value) -> FlowNode {
        FlowNode {
            id: id.into(),
            kind,
            position: FlowPosition { x: 0.0, y: 0.0 },
            data,
            style: json!({}),
            width: None,
            height: None,
        }
    }

    fn flow_edge(id: &str, source: &str, target: &str) -> FlowEdge {
        FlowEdge {
            id: id.into(),
            source: source.into(),
            target: target.into(),
            source_handle: Some("output".into()),
            target_handle: Some("input".into()),
        }
    }

    #[test]
    fn parses_a_successful_direct_variable_response() {
        let response = parse_runner_response(
            br#"{"ok":true,"output":{"total":42},"stdout":"logged\n","stderr":""}"#,
        )
        .expect("response should parse");

        assert!(response.ok);
        assert_eq!(response.output, Some(json!({ "total": 42 })));
        assert_eq!(response.stdout, "logged\n");
    }

    #[test]
    fn rejects_non_json_runner_output() {
        assert!(parse_runner_response(b"not json").is_err());
    }

    #[test]
    fn requires_an_absolute_existing_project_directory() {
        assert!(project_path("relative-project").is_err());
        assert!(project_path("/definitely/not/a/cathopper/project").is_err());
    }

    #[test]
    fn parses_input_node_json() {
        let node = FlowNode {
            id: "input".into(),
            kind: FlowNodeKind::InputNode,
            position: FlowPosition { x: 0.0, y: 0.0 },
            data: json!({ "value": "{\"count\": 2}" }),
            style: json!({}),
            width: None,
            height: None,
        };

        assert_eq!(input_value(&node), Ok(json!({ "count": 2 })));
    }

    #[test]
    fn evaluates_nodes_in_edge_order() {
        let nodes = vec![
            flow_node("output", FlowNodeKind::OutputNode, json!({})),
            flow_node(
                "python",
                FlowNodeKind::PythonNode,
                json!({ "code": "double" }),
            ),
            flow_node("input", FlowNodeKind::InputNode, json!({ "value": "3" })),
        ];
        let edges = vec![
            flow_edge("input-python", "input", "python"),
            flow_edge("python-output", "python", "output"),
        ];

        let values = evaluate_flow(nodes, edges, None, None, |source, input| {
            assert_eq!(source, "double");
            Ok(json!(input.as_i64().unwrap() * 2))
        })
        .expect("connected flow should run");

        assert_eq!(values["output"], json!(6));
    }

    #[test]
    fn rejects_cycles_before_execution() {
        let nodes = vec![
            flow_node("first", FlowNodeKind::OutputNode, json!({})),
            flow_node("second", FlowNodeKind::OutputNode, json!({})),
        ];
        let edges = vec![
            flow_edge("first-second", "first", "second"),
            flow_edge("second-first", "second", "first"),
        ];

        let failure = match execution_plan(nodes, edges) {
            Ok(_) => panic!("cycle should be rejected"),
            Err(failure) => failure,
        };

        assert_eq!(failure.error, "Flow contains a cycle.");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(FlowStore::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            run_python,
            get_flow,
            replace_flow,
            run_flow
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
