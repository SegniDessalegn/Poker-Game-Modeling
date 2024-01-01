import random
import numpy as np

_N_ACTIONS = 3
_N_CARDS = 3
PUBLIC_CARD = random.randint(0, 2)

terminals = set([
    "iicccc",
    "iicccrc",
    "iicccrf",
    "iicccf",
    "iiccrc",
    "iiccrrc",
    "iicccr",
    "iiccrrf",
    "iiccrf",
    "iiccf",
    "iicrcc",
    "iicrcrc",
    "iicrcrf",
    "iicrcf",
    "iicrrc",
    "iicrrf",
    "iicrf",
    "iicf",
    "iirccc",
    "iirccrc",
    "iirccrf",
    "iirccf",
    "iircrc",
    "iircrf",
    "iircf",
    "iirrcc",
    "iirrcf",
    "iirrcc",
    "iirrf",
    "iirf",
    "iif",
])

def main():
    """
    Run iterations of counterfactual regret minimization algorithm.
    """
    i_map = {}  # map of information sets
    n_iterations = 10000
    expected_game_value = 0

    for i in range(n_iterations - 1):
        expected_game_value += cfr(i_map, "", 0, 0)
        for _, v in i_map.items():
            v.next_strategy()
        print("iteration", i, "expected value:", expected_game_value / n_iterations)

    expected_game_value /= n_iterations

    display_results(expected_game_value, i_map)


def cfr(i_map, history="", card_1=-1, card_2=-1, pr_1=1, pr_2=1, pr_c=1, bet = 0):
    """
    Counterfactual regret minimization algorithm.
    """
    if is_chance_node(history):
        return chance_util(i_map)

    if is_terminal(history):
        return terminal_util(history, card_1, card_2)

    n = len(history)
    is_player_1 = n % 2 == 0
    info_set = get_info_set(i_map, card_1 if is_player_1 else card_2, history)

    strategy = info_set.strategy
    if is_player_1:
        info_set.reach_pr += pr_1
    else:
        info_set.reach_pr += pr_2

    action_utils = np.zeros(_N_ACTIONS)

    for i, action in enumerate(["c", "r", "f"]):
        if action == "r" and bet == 4:
            continue

        next_history = history + action
        if is_player_1:
            action_utils[i] = -1 * cfr(i_map, next_history,
                                       card_1, card_2,
                                       pr_1 * strategy[i], pr_2, pr_c, bet + (2 if action == "r" else 0))
        else:
            action_utils[i] = -1 * cfr(i_map, next_history,
                                       card_1, card_2,
                                       pr_1, pr_2 * strategy[i], pr_c, bet + (2 if action == "r" else 0))

    util = sum(action_utils * strategy)
    regrets = action_utils - util
    if is_player_1:
        info_set.regret_sum += pr_2 * pr_c * regrets
    else:
        info_set.regret_sum += pr_1 * pr_c * regrets

    return util

def is_chance_node(history):
    """
    Determine if we are at a chance node based on tree history.
    """
    return history == ""

def chance_util(i_map):
    expected_value = 0
    n_possibilities = 6
    for i in range(_N_CARDS):
        for j in range(_N_CARDS):
            if i != j:
                expected_value += cfr(i_map, "ii", i, j,
                                    1, 1, 1/n_possibilities)
    return expected_value / n_possibilities

def is_terminal(history):
    """
    Returns true if the history is a terminal history.
    """
    return history in terminals

def terminal_util(history, card_1, card_2):
    card_val = {"J": 1, "Q": 2, "K": 3}

    n = len(history)
    card_player = 1 if n % 2 == 0 else -1

    net = get_pot(history, card_player)

    if history[-1] == "f":
        return -card_player * net

    elif card_player == 1:
        if card_1 == PUBLIC_CARD:
            return card_player * net
        elif card_2 == PUBLIC_CARD:
            return -card_player * net
        else:
            return card_player * (1 if card_val[card_str(card_1)] > card_val[card_str(card_2)] else -1) * net
    else:
        if card_2 == PUBLIC_CARD:
            return -card_player * net
        elif card_1 == PUBLIC_CARD:
            return card_player * net
        else:
            return -(card_player * (1 if card_val[card_str(card_1)] > card_val[card_str(card_2)] else -1) * net)


def get_pot(history, turn):
    bet = {0:0, 1:0}
    prev = 1
    player = 0
    pot = 1
    bet_round = 1
    for op in history[2:]:
        if op == "r":
            bet[player] += prev - ((bet_round * 2) - bet[player])
            prev += 2
            bet_round += 1
        elif op == "c":
            bet[player] += prev - bet[player]

        player = (player + 1) % 2

    return pot + (bet[0] if turn == -1 else bet[1])


def card_str(card):
    combs = ["J", "Q", "K"]
    return combs[card]

def get_info_set(i_map, card, history):
    """
    Retrieve information set from dictionary.
    """
    key = card_str(card) + " " + history
    info_set = None

    if key not in i_map:
        info_set = InformationSet(key)
        i_map[key] = info_set
        return info_set

    return i_map[key]


class InformationSet():
    def __init__(self, key):
        self.key = key
        self.regret_sum = np.zeros(_N_ACTIONS)
        self.strategy_sum = np.zeros(_N_ACTIONS)
        self.strategy = np.repeat(1/_N_ACTIONS, _N_ACTIONS)
        self.reach_pr = 0
        self.reach_pr_sum = 0

    def next_strategy(self):
        self.strategy_sum += self.reach_pr * self.strategy
        self.strategy = self.calc_strategy()
        self.reach_pr_sum += self.reach_pr
        self.reach_pr = 0

    def calc_strategy(self):
        strategy = self.make_positive(self.regret_sum)
        total = sum(strategy)
        if total > 0:
            strategy = strategy / total
        else:
            n = _N_ACTIONS
            strategy = np.repeat(1/n, n)

        return strategy

    def get_average_strategy(self):
        strategy = self.strategy_sum / self.reach_pr_sum

        strategy = np.where(strategy < 0.001, 0, strategy)

        total = sum(strategy)
        strategy /= total

        return strategy

    def make_positive(self, x):
        return np.where(x > 0, x, 0)

    def __str__(self):
        strategies = ['{:03.2f}'.format(x)
                      for x in self.get_average_strategy()]
        card = self.key.ljust(6)
        if len((card.split())[-1]) > 3:
            card = card[0] + card_str(PUBLIC_CARD) + card[1:]
        return '{} {}'.format(card, strategies)


def display_results(ev, i_map):
    print()
    print("==== notation ====")
    print("i => initial")
    print("==================")
    print()

    print('player 1 expected value: {}'.format(ev))
    print('player 2 expected value: {}'.format(-1 * ev))
    print()

    print("******* PUBLIC CARD ********")
    print("----------->", card_str(PUBLIC_CARD), "<------------")
    print("****************************")
    print()
    print('player 1 strategies:')
    sorted_items = sorted(i_map.items(), key=lambda x: x[0])

    for _, v in filter(lambda x: len(x[0]) % 2 == 0, sorted_items):
        print(v)
    print()
    print('player 2 strategies:')
    for _, v in filter(lambda x: len(x[0]) % 2 == 1, sorted_items):
        print(v)


if __name__ == "__main__":
    main()
