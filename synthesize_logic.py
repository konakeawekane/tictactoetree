from __future__ import annotations

import argparse
import csv
import functools
import itertools
import json
import random
import struct
import time
from pathlib import Path

TERMINALS = 9


def load_source(path: Path) -> tuple[list[tuple[int, ...]], list[int]]:
    decisions: dict[tuple[int, ...], int] = {}
    with path.open(newline="", encoding="utf-8") as source:
        for row in csv.DictReader(source):
            target = int(row["target_move"])
            if target < 0:
                continue
            player = int(row["player"])
            state = tuple(int(row[f"b{i}"]) * player for i in range(9))
            previous = decisions.setdefault(state, target)
            if previous != target:
                raise ValueError(f"Conflicting decisions for {state}")
    ordered = sorted(decisions.items())
    return [state for state, _ in ordered], [target for _, target in ordered]


def compact(root: int, nodes: list[tuple[int, tuple[int, int, int]]]):
    reachable: set[int] = set()

    def visit(reference: int) -> None:
        if reference < TERMINALS:
            return
        index = reference - TERMINALS
        if index in reachable:
            return
        reachable.add(index)
        for child in nodes[index][1]:
            visit(child)

    visit(root)
    ordered = sorted(reachable)
    remap = {old: TERMINALS + new for new, old in enumerate(ordered)}

    def mapped(reference: int) -> int:
        return reference if reference < TERMINALS else remap[reference - TERMINALS]

    result = [
        (nodes[index][0], tuple(mapped(child) for child in nodes[index][1]))
        for index in ordered
    ]
    return mapped(root), result


def build_dag(states, targets, order: tuple[int, ...]):
    nodes: list[tuple[int, tuple[int, int, int]]] = []
    unique: dict[tuple[int, int, int, int], int] = {}

    def build(rows: tuple[int, ...], level: int) -> int:
        labels = {targets[index] for index in rows}
        if len(labels) == 1:
            return next(iter(labels))
        if level == len(order):
            raise ValueError("Conflicting labels for an identical board")
        cell = order[level]
        groups = tuple(
            tuple(index for index in rows if states[index][cell] == value)
            for value in (-1, 0, 1)
        )
        children: list[int | None] = [
            build(group, level + 1) if group else None for group in groups
        ]
        existing = [child for child in children if child is not None]
        default = min(existing)
        resolved = tuple(default if child is None else child for child in children)
        if resolved[0] == resolved[1] == resolved[2]:
            return resolved[0]
        key = (cell, *resolved)
        reference = unique.get(key)
        if reference is None:
            reference = TERMINALS + len(nodes)
            unique[key] = reference
            nodes.append((cell, resolved))
        return reference

    root = build(tuple(range(len(states))), 0)
    return compact(root, nodes)


def packed_bytes(node_count: int) -> int:
    reference_bytes = 1 if node_count + TERMINALS <= 256 else 2
    return 12 + node_count * (1 + 3 * reference_bytes)


def build_adaptive_dag(states, targets):
    @functools.cache
    def signatures(expression) -> frozenset:
        if isinstance(expression, int):
            return frozenset()
        result = {expression}
        for child in expression[1:]:
            result.update(signatures(child))
        return frozenset(result)

    @functools.cache
    def solve(rows: tuple[int, ...], remaining: tuple[int, ...]):
        labels = {targets[index] for index in rows}
        if len(labels) == 1:
            return next(iter(labels))
        best = None
        best_cost = None
        for cell in remaining:
            next_remaining = tuple(value for value in remaining if value != cell)
            groups = tuple(
                tuple(index for index in rows if states[index][cell] == value)
                for value in (-1, 0, 1)
            )
            children = [solve(group, next_remaining) if group else None for group in groups]
            existing = [child for child in children if child is not None]
            default = min(existing, key=repr)
            resolved = tuple(default if child is None else child for child in children)
            expression = (
                resolved[0] if resolved[0] == resolved[1] == resolved[2]
                else (cell, *resolved)
            )
            cost = len(signatures(expression))
            if best_cost is None or cost < best_cost:
                best, best_cost = expression, cost
        return best

    expression = solve(tuple(range(len(states))), tuple(range(9)))
    nodes: list[tuple[int, tuple[int, int, int]]] = []
    emitted: dict[tuple, int] = {}

    def emit(value) -> int:
        if isinstance(value, int):
            return value
        reference = emitted.get(value)
        if reference is not None:
            return reference
        children = tuple(emit(child) for child in value[1:])
        reference = TERMINALS + len(nodes)
        emitted[value] = reference
        nodes.append((value[0], children))
        return reference

    root = emit(expression)
    return compact(root, nodes)


def infer(model: dict, state: tuple[int, ...]) -> int:
    reference = model["root"]
    while reference >= TERMINALS:
        node = model["nodes"][reference - TERMINALS]
        reference = node["children"][state[node["cell"]] + 1]
    return reference


def verify(model: dict, states, targets) -> None:
    mistakes = [
        (state, target, infer(model, state))
        for state, target in zip(states, targets)
        if infer(model, state) != target
    ]
    if mistakes:
        raise RuntimeError(f"Logic DAG has {len(mistakes)} errors: {mistakes[:3]}")


WIN_LINES = (
    (0, 1, 2), (3, 4, 5), (6, 7, 8),
    (0, 3, 6), (1, 4, 7), (2, 5, 8),
    (0, 4, 8), (2, 4, 6),
)


def worst_case_result(model: dict, ai: int) -> int:
    cache: dict[tuple[tuple[int, ...], int], int] = {}

    def solve(board: tuple[int, ...], player: int) -> int:
        key = (board, player)
        if key in cache:
            return cache[key]
        for a, b, c in WIN_LINES:
            if board[a] != 0 and board[a] == board[b] == board[c]:
                return 1 if board[a] == ai else -1
        legal = [index for index, value in enumerate(board) if value == 0]
        if not legal:
            return 0
        if player == ai:
            state = tuple(value * player for value in board)
            selected = infer(model, state)
            if selected not in legal:
                return -1
            next_board = list(board)
            next_board[selected] = player
            result = solve(tuple(next_board), -player)
        else:
            outcomes = []
            for selected in legal:
                next_board = list(board)
                next_board[selected] = player
                outcomes.append(solve(tuple(next_board), -player))
            result = min(outcomes)
        cache[key] = result
        return result

    return solve((0,) * 9, 1)


def make_model(root, nodes, order, source_decisions) -> dict:
    return {
        "format": "sparse-ternary-logic-policy-v1",
        "synthesis": (
            "reduced ordered multilevel decision DAG with don't-cares"
            if order
            else "adaptive multilevel decision DAG with don't-cares"
        ),
        "input": "9 cells from current-player perspective: own=1, opponent=-1, empty=0",
        "output": "terminal board index 0..8",
        "variable_order": list(order),
        "root": root,
        "nodes": [
            {"cell": cell, "children": list(children)} for cell, children in nodes
        ],
        "node_count": len(nodes),
        "edge_count": len(nodes) * 3,
        "source_decisions": source_decisions,
        "source_accuracy": 1.0,
        "packed_bytes": packed_bytes(len(nodes)),
    }


def write_model(model: dict, output: Path) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(model, separators=(",", ":")), encoding="utf-8")
    with output.with_suffix(".logic").open("wb") as destination:
        destination.write(b"TTTLOG1\0")
        destination.write(struct.pack("<HH", len(model["nodes"]), model["root"]))
        reference_format = "B" if len(model["nodes"]) + TERMINALS <= 256 else "H"
        for node in model["nodes"]:
            destination.write(
                struct.pack("<B" + reference_format * 3, node["cell"], *node["children"])
            )


def search(states, targets, seconds: float, seed: int):
    rng = random.Random(seed)
    deadline = time.monotonic() + seconds
    cache: dict[tuple[int, ...], tuple[int, list]] = {}
    best = None
    attempts = 0

    def evaluate(order: tuple[int, ...]):
        nonlocal best, attempts
        result = cache.get(order)
        if result is None:
            result = build_dag(states, targets, order)
            cache[order] = result
        attempts += 1
        if best is None or len(result[1]) < len(best[2]):
            best = (order, result[0], result[1])
            print(
                f"new_best nodes={len(result[1])} bytes={packed_bytes(len(result[1]))} "
                f"order={order} attempts={attempts}",
                flush=True,
            )
        return len(result[1])

    starts = [tuple(range(9)), tuple(reversed(range(9)))]
    while time.monotonic() < deadline:
        current = starts.pop(0) if starts else tuple(rng.sample(range(9), 9))
        score = evaluate(current)
        improved = True
        while improved and time.monotonic() < deadline:
            improved = False
            choices = []
            for left, right in itertools.combinations(range(9), 2):
                proposal = list(current)
                proposal[left], proposal[right] = proposal[right], proposal[left]
                proposal = tuple(proposal)
                choices.append((evaluate(proposal), proposal))
                if time.monotonic() >= deadline:
                    break
            if choices:
                candidate_score, candidate = min(choices)
                if candidate_score < score:
                    current, score, improved = candidate, candidate_score, True
    assert best is not None
    print(f"search_complete attempts={attempts} cached_orders={len(cache)}")
    return best


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--data", type=Path, default=Path("data/tictactoe_games.csv"))
    parser.add_argument("--output", type=Path, default=Path("artifacts/logic_policy.json"))
    parser.add_argument("--search-seconds", type=float, default=30.0)
    parser.add_argument("--seed", type=int, default=23)
    args = parser.parse_args()

    states, targets = load_source(args.data)
    print("building adaptive multilevel DAG...", flush=True)
    adaptive_root, adaptive_nodes = build_adaptive_dag(states, targets)
    print(
        f"adaptive nodes={len(adaptive_nodes)} bytes={packed_bytes(len(adaptive_nodes))}",
        flush=True,
    )
    order, ordered_root, ordered_nodes = search(
        states, targets, args.search_seconds, args.seed
    )
    if len(adaptive_nodes) <= len(ordered_nodes):
        order, root, nodes = (), adaptive_root, adaptive_nodes
    else:
        root, nodes = ordered_root, ordered_nodes
    model = make_model(root, nodes, order, len(states))
    verify(model, states, targets)
    gameplay = {"X": worst_case_result(model, 1), "O": worst_case_result(model, -1)}
    model["worst_case_result"] = gameplay
    if min(gameplay.values()) < 0:
        raise RuntimeError(f"Exact logic policy can be defeated: {gameplay}")
    write_model(model, args.output)
    print(
        f"verified accuracy=100% nodes={len(nodes)} bytes={model['packed_bytes']} "
        f"worst_case={gameplay} output={args.output}"
    )


if __name__ == "__main__":
    main()
