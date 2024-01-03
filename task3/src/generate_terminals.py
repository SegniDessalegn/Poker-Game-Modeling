
"""
Generates terminal histories in a full exhaustive game representation
It also generates specifically for first round and second round terminals

We can achieve this using DFS (Depth First Search) on the possible operations
-> There are three actions (c, r, f)
-> The game has two rounds, unless 'f' action is choosen
-> There can be at most 2 rounds of bet
"""


actions = ["c", "r", "f"]

def complete_terminal(history):
    history = "".join(history)
    if history[-1] == "f":
        return [history]

    if history[-1] == "r":
        return [history + "c", history + "f"]

    return [history]


def gen(last_action, play_count, bet, history):

    if last_action == "f" or play_count == 0:
        for terminal in complete_terminal(history):
            print(terminal)
        return

    for action in actions:
        new_bet = bet
        new_play_count = play_count - 1
        if action == "r":
            if bet == 4:
                continue
            new_bet = bet + 2

        history.append(action)
        gen(action, new_play_count, new_bet, history)
        history.pop()


def generate():
    print()
    print("first round terminals")
    gen("", 2, 0, [])
    print()
    print("second round terminals")
    gen("", 4, 0, [])


generate()
