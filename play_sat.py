from __future__ import annotations

import argparse
import json
from pathlib import Path


NAMES = ("TL", "TM", "TR", "ML", "MM", "MR", "BL", "BM", "BR")
WIN_LINES = (
    (0, 1, 2),
    (3, 4, 5),
    (6, 7, 8),
    (0, 3, 6),
    (1, 4, 7),
    (2, 5, 8),
    (0, 4, 8),
    (2, 4, 6),
)


def winner(board: list[int]) -> int:
    for first, second, third in WIN_LINES:
        if board[first] != 0 and board[first] == board[second] == board[third]:
            return board[first]
    return 0


def display(board: list[int]) -> None:
    symbols = {1: "X", -1: "O", 0: "_"}
    print(
        f"{symbols[board[0]]}|{symbols[board[1]]}|{symbols[board[2]]}\n"
        f"-+-+-\n{symbols[board[3]]}|{symbols[board[4]]}|{symbols[board[5]]}\n"
        f"-+-+-\n{symbols[board[6]]}|{symbols[board[7]]}|{symbols[board[8]]}"
    )


def model_move(model: dict, board: list[int], player: int) -> int:
    perspective = [value * player for value in board]
    if model.get("format") == "sparse-ternary-logic-policy-v1":
        reference = model["root"]
        while reference >= 9:
            node = model["nodes"][reference - 9]
            reference = node["children"][perspective[node["cell"]] + 1]
        if board[reference] != 0:
            raise RuntimeError("Logic policy selected an occupied square")
        return reference

    values: list[int] = []
    for value in perspective:
        values.extend(1 if category == value else 0 for category in (-1, 0, 1))
    values.extend((1,) * 7)

    for layer_index, layer in enumerate(model["layers"]):
        sums = [
            sum(weight * value for weight, value in zip(row, values)) + bias
            for row, bias in zip(layer["weight"], layer["bias"])
        ]
        values = [1 if value >= 0 else 0 for value in sums] if layer_index < len(model["layers"]) - 1 else sums

    legal = [index for index, value in enumerate(board) if value == 0]
    return max(legal, key=lambda index: (values[index], -index))


def human_move(board: list[int]) -> int | None:
    while True:
        text = input("Your move (TL..BR, or Q): ").strip().upper()
        if text == "Q":
            return None
        if text in NAMES:
            move = NAMES.index(text)
            if board[move] == 0:
                return move
        print("Invalid or occupied square.")


def play_game(model: dict, computer: int) -> bool:
    board = [0] * 9
    player = 1
    while True:
        print()
        display(board)
        if player == computer:
            move = model_move(model, board, player)
            print(f"Computer plays {NAMES[move]}.")
        else:
            move = human_move(board)
            if move is None:
                return False
        board[move] = player
        result = winner(board)
        if result or all(board):
            print()
            display(board)
            print("Computer wins!" if result == computer else "You win!" if result else "Cat's game!")
            return True
        player = -player


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--model", type=Path, default=Path("artifacts/logic_policy.json")
    )
    args = parser.parse_args()
    model = json.loads(args.model.read_text(encoding="utf-8"))
    if model.get("source_accuracy") != 1.0:
        raise ValueError("Refusing to play with an unverified model")

    print("Positions: TL TM TR / ML MM MR / BL BM BR")
    while True:
        choice = input("Who starts? [C]omputer, [H]uman, [Q]uit: ").strip().upper()
        if choice == "Q":
            break
        if choice not in {"C", "H"}:
            continue
        if not play_game(model, 1 if choice == "C" else -1):
            break


if __name__ == "__main__":
    main()
