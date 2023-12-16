
initial_pot = 60

operations = {
    "x": ["x", "b"],
    "b": ["r", "c", "f"],
    "c": ["c", "f", "r"],
    "f": ["r", "c", "f"],
    "r": ["c", "f"]
}

def calculate(seq):
    _sum = 0

    prev = 0
    for op in seq:
        if op == "b" and prev != 10:
            prev = 10
            _sum += 10
        elif op == "r" and prev != 20:
            prev = 20
            _sum += 20
        elif op == "c":
            _sum += prev

    return initial_pot + _sum


def choose_next_player(curr_player, fold):
    next_player = curr_player + 1
    next_player %= 3
    while next_player in fold:
        next_player += 1
        next_player %= 3

    return next_player


def generate(curr_player, prev_op, player_stop, seq, fold, turn = 1, raised = False):

    if curr_player == player_stop or turn == 3 or len(fold) == 2:
        print(":".join(["b10" if s == "b" else "r20" if s == "r" else s for s in seq]) + ",", calculate(seq))
        return

    next_player = choose_next_player(curr_player, fold)

    for next_op in operations[prev_op]:
        if next_op == "r" and raised:
            continue

        if next_op == "f":
            fold.add(curr_player)

        seq.append(next_op)
        next_player_stop = player_stop
        if next_op == "r" or next_op == "b":
            next_player_stop = curr_player

        new_raised = raised
        if next_op == "r":
            new_raised = True
        generate(next_player, next_op, next_player_stop, seq, fold, turn + (1 if next_player < curr_player else 0), new_raised)

        if next_op == "f":
            fold.discard(curr_player)

        seq.pop()


generate(1, "x", 0, ["x"], set())
generate(1, "b", 0, ["b"], set())
