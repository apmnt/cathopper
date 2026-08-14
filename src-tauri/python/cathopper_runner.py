"""Execute one Cathopper Python node and emit exactly one JSON response."""

import contextlib
import io
import json
import runpy
import sys
import traceback
from collections.abc import Callable
from typing import TypedDict, cast

type JsonValue = (
    None | bool | int | float | str | list["JsonValue"] | dict[str, "JsonValue"]
)
type NodeFunction = Callable[[JsonValue], tuple[JsonValue]]


class RunnerRequest(TypedDict):
    input: JsonValue
    sourcePath: str


class RunnerError(TypedDict):
    kind: str
    message: str
    traceback: str


class SuccessResponse(TypedDict):
    ok: bool
    output: JsonValue
    stdout: str
    stderr: str


class FailureResponse(TypedDict):
    ok: bool
    stdout: str
    stderr: str
    error: RunnerError


type RunnerResponse = SuccessResponse | FailureResponse


def success_response(output: JsonValue, stdout: str, stderr: str) -> SuccessResponse:
    return {"ok": True, "output": output, "stdout": stdout, "stderr": stderr}


def failure_response(
    kind: str,
    message: str,
    stdout: str = "",
    stderr: str = "",
    traceback_text: str = "",
) -> FailureResponse:
    return {
        "ok": False,
        "stdout": stdout,
        "stderr": stderr,
        "error": {"kind": kind, "message": message, "traceback": traceback_text},
    }


def parse_request(request: JsonValue) -> RunnerRequest:
    if not isinstance(request, dict):
        raise TypeError("Runner request must be a JSON object")

    node_input = request["input"]
    source_path = request["sourcePath"]
    if not isinstance(source_path, str):
        raise TypeError("Runner request sourcePath must be a string")
    return {"input": node_input, "sourcePath": source_path}


def request_values(request: RunnerRequest) -> tuple[JsonValue, str]:
    return request["input"], request["sourcePath"]


def execute_node(node_input: JsonValue, source_path: str) -> RunnerResponse:
    stdout = io.StringIO()
    stderr = io.StringIO()

    try:
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            namespace = cast(
                dict[str, object],
                runpy.run_path(source_path, run_name="__cathopper_node__"),
            )
            run = namespace.get("run")
            if not callable(run):
                raise TypeError("Python node must define run(input)")
            result = cast(NodeFunction, run)(node_input)

        if not isinstance(result, tuple) or len(result) != 1:
            raise TypeError("run(input) must return a one-item tuple")
        output = cast(JsonValue, result[0])
        _ = json.dumps(output, allow_nan=False)
        return success_response(output, stdout.getvalue(), stderr.getvalue())
    except Exception as error:
        return failure_response(
            "execution",
            str(error),
            stdout.getvalue(),
            stderr.getvalue(),
            traceback.format_exc(),
        )


def main() -> None:
    try:
        request = parse_request(cast(JsonValue, json.load(sys.stdin)))
        node_input, source_path = request_values(request)
    except (KeyError, TypeError, json.JSONDecodeError) as error:
        payload = failure_response("request", f"Invalid runner request: {error}")
    else:
        payload = execute_node(node_input, source_path)

    sys.__stdout__.write(json.dumps(payload, allow_nan=False) + "\n")


if __name__ == "__main__":
    main()
