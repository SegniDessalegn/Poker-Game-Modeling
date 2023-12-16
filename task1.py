

"""
We can use backtracking to generate all the possible sequences
"""

class Game:
    def __init__(self, initial_pot = 60):
        self.initial_pot = initial_pot
        self.operations = {
            "x": ["x", "b"],
            "b": ["r", "c", "f"],
            "c": ["c", "f", "r"],
            "f": ["r", "c", "f"],
            "r": ["c", "f"]
        }

    def generate(self):
        self.generate_seq(1, "x", 0, ["x"], set())
        self.generate_seq(1, "b", 0, ["b"], set())

    def calculate(self, seq):
        bet = {0:0, 1:0, 2:0}
        prev = 0
        player = 0
        for op in seq:
            if op == "b":
                prev = 10
                bet[player] += 10 - bet[player]
            elif op == "r":
                prev = 20
                bet[player] += 20 - bet[player]
            elif op == "c":
                bet[player] += prev - bet[player]

            player = (player + 1) % 3

        return self.initial_pot + sum(bet.values())


    def choose_next_player(self, curr_player, fold):
        next_player = curr_player + 1
        next_player %= 3
        while next_player in fold:
            next_player += 1
            next_player %= 3

        return next_player


    def generate_seq(self, curr_player, prev_op, player_stop, seq, fold, raised = False):
        if curr_player == player_stop or len(fold) == 2:
            print(":".join(["b10" if s == "b" else "r20" if s == "r" else s for s in seq]) + ",", self.calculate(seq))
            return

        next_player = self.choose_next_player(curr_player, fold)

        for next_op in self.operations[prev_op]:
            if next_op == "r" and raised:
                continue

            if next_op == "f":
                fold.add(curr_player)

            next_player_stop = player_stop
            if next_op == "r" or next_op == "b":
                next_player_stop = curr_player

            new_raised = raised
            if next_op == "r":
                new_raised = True

            seq.append(next_op)

            self.generate_seq(next_player, next_op, next_player_stop, seq, fold, new_raised)

            seq.pop()
            if next_op == "f":
                fold.discard(curr_player)


game = Game()
game.generate()
