
actions = ["r", "c", "f"]

def complete_terminal(history):
    history = "".join(history)
    if history[-1] == "r":
        return [history + "cc", history + "cf", history + "f"]

    if len(history) >= 2 and history[-2] == "r":
        return [history + "c", history + "f"]

    return [history]


def gen(last_action, play_count, bet, history):

    if last_action == "f" or play_count > 4:
        for terminal in complete_terminal(history):
            print(terminal)
        return

    for action in actions:
        new_bet = bet
        new_play_count = play_count + 1
        if action == "r":
            if bet == 4:
                continue
            new_bet = bet + 2

        history.append(action)
        gen(action, new_play_count, new_bet, history)
        history.pop()


gen("", 1, 0, [])
