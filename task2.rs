use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt;

const _N_ACTIONS: i32 = 2;
const _N_CARDS: i32 = 3;

fn main() {
    let mut i_map: HashMap<String, InformationSet> = HashMap::new();
    let mut expected_game_value = 0.0; // Make it mutable
    let n_iterations = 10000;

    for _ in 0..n_iterations {
        expected_game_value += cfr(&mut i_map, "", -1, -1, 1.0, 1.0, 1.0);
        for v in i_map.values_mut() {
            v.next_strategy();
        }
    }

    expected_game_value /= n_iterations as f64;

    display_results(expected_game_value, &i_map);
}

fn cfr(
    i_map: &mut HashMap<String, InformationSet>,
    history: &str,
    card_1: i32,
    card_2: i32,
    pr_1: f64,
    pr_2: f64,
    pr_c: f64,
) -> f64 {
    if is_chance_node(history) {
        return chance_util(i_map);
    }

    if is_terminal(history) {
        return terminal_util(history, card_1, card_2) as f64;
    }

    let n = history.len();
    let is_player_1 = n % 2 == 0;
    let mut info_set = get_info_set(i_map, if is_player_1 { card_1 } else { card_2 }, history);

    let strategy = &info_set.strategy;
    if is_player_1 {
        info_set.reach_pr += pr_1;
    } else {
        info_set.reach_pr += pr_2;
    }

    // Counterfactual utility per action.
    let mut action_utils = vec![0.0; _N_ACTIONS as usize];

    for (i, action) in ["c", "b"].iter().enumerate() {
        let next_history = format!("{}{}", history, action);
        if is_player_1 {
            action_utils[i] =
                -1.0 * cfr(i_map, &next_history, card_1, card_2, pr_1 * strategy[i], pr_2, pr_c);
        } else {
            action_utils[i] =
                -1.0 * cfr(i_map, &next_history, card_1, card_2, pr_1, pr_2 * strategy[i], pr_c);
        }
    }

    // Utility of information set.
    let util = action_utils.iter().zip(strategy.iter()).map(|(&a, &b)| a * b).sum();
    let regrets: Vec<_> = action_utils.iter().map(|&a| a - util).collect();

    if is_player_1 {
        info_set.regret_sum.iter_mut().zip(regrets.iter()).for_each(|(r, &regret)| {
            *r += pr_2 * pr_c * regret;
        });
    } else {
        info_set.regret_sum.iter_mut().zip(regrets.iter()).for_each(|(r, &regret)| {
            *r += pr_1 * pr_c * regret;
        });
    }

    util
}

fn is_chance_node(history: &str) -> bool {
    history == ""
}

fn chance_util(i_map: &mut HashMap<String, InformationSet>) -> f64 {
    let mut expected_value = 0.0;
    let n_possibilities = 6;

    for i in 0.._N_CARDS {
        for j in 0.._N_CARDS {
            if i != j {
                expected_value +=
                    cfr(i_map, "rr", i, j, 1.0, 1.0, 1.0 / n_possibilities as f64);
            }
        }
    }

    expected_value / n_possibilities as f64
}

fn is_terminal(history: &str) -> bool {
    let possibilities = ["rrcc", "rrcbc", "rrcbb", "rrbc", "rrbb"];
    possibilities.contains(&history)
}

fn terminal_util(history: &str, card_1: i32, card_2: i32) -> i32 {
    let n = history.len();
    let card_player = if n % 2 == 0 { card_1 } else { card_2 };
    let card_opponent = if n % 2 == 0 { card_2 } else { card_1 };

    match history {
        "rrcbc" | "rrbc" => 1, // Last player folded. The current player wins.
        "rrcc" => {
            // Showdown with no bets
            if card_player > card_opponent {
                1
            } else {
                -1
            }
        }
        "rrcbb" | "rrbb" => {
            // Showdown with 1 bet
            if card_player > card_opponent {
                2
            } else {
                -2
            }
        }
        _ => panic!("Invalid history"),
    }
}

fn card_str(card: usize) -> String {
    match card {
        0 => String::from("J"),
        1 => String::from("Q"),
        _ => String::from("K"),
    }
}

fn get_info_set(i_map: &mut HashMap<String, InformationSet>, card: i32, history: &str) -> InformationSet {
    let key = format!("{} {}", card_str(card as usize), history);
    let info_set = i_map.entry(key.clone()).or_insert_with(|| InformationSet::new(key.clone()));
    info_set.clone()
}

struct InformationSet {
    key: String,
    regret_sum: [f64; _N_ACTIONS as usize],
    strategy_sum: [f64; _N_ACTIONS as usize],
    strategy: [f64; _N_ACTIONS as usize],
    reach_pr: f64,
    reach_pr_sum: f64,
}

impl InformationSet {
    fn new(key: String) -> Self {
        Self {
            key,
            regret_sum: [0.0; _N_ACTIONS as usize],
            strategy_sum: [0.0; _N_ACTIONS as usize],
            strategy: [1.0 / _N_ACTIONS as f64; _N_ACTIONS as usize],
            reach_pr: 0.0,
            reach_pr_sum: 0.0,
        }
    }

    fn next_strategy(&mut self) {
        for i in 0.._N_ACTIONS as usize {
            self.strategy_sum[i] += self.strategy[i] * self.reach_pr;
        }
        self.strategy = self.calc_strategy();
        self.reach_pr_sum += self.reach_pr;
        self.reach_pr = 0.0;
    }

    fn calc_strategy(&self) -> [f64; _N_ACTIONS as usize] {
        let strategy = self.make_positive(&self.regret_sum);
        let total: f64 = strategy.iter().sum();
        if total > 0.0 {
            strategy.iter().map(|&x| x / total).collect::<Vec<_>>().try_into().unwrap()
        } else {
            [1.0 / _N_ACTIONS as f64; _N_ACTIONS as usize]
        }
    }

    fn get_average_strategy(&self) -> [f64; _N_ACTIONS as usize] {
        let mut strategy = self.strategy_sum;
        for x in strategy.iter_mut() {
            if *x < 0.001 {
                *x = 0.0;
            }
        }
        let total: f64 = strategy.iter().sum();
        for x in strategy.iter_mut() {
            *x /= total;
        }
        strategy
    }

    fn make_positive(&self, x: &[f64; _N_ACTIONS as usize]) -> Vec<f64> {
        x.iter().map(|&val| if val > 0.0 { val } else { 0.0 }).collect()
    }
}

impl Clone for InformationSet {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            regret_sum: self.regret_sum.clone(),
            strategy_sum: self.strategy_sum.clone(),
            strategy: self.strategy.clone(),
            reach_pr: self.reach_pr,
            reach_pr_sum: self.reach_pr_sum,
        }
    }
}

impl fmt::Display for InformationSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let strategies: Vec<String> = self
            .get_average_strategy()
            .iter()
            .map(|&x| format!("{:03.2}", x))
            .collect();
        write!(f, "{} {}", self.key, strategies.join(" "))
    }
}

fn display_results(ev: f64, i_map: &HashMap<String, InformationSet>) {
    println!("player 1 expected value: {}", ev);
    println!("player 2 expected value: {}", -1.0 * ev);

    println!();
    println!("player 1 strategies:");
    let sorted_items: Vec<_> = i_map.iter().filter(|(key, _)| key.len() % 2 == 0).collect();
    for (_, v) in sorted_items {
        println!("{}", v);
    }

    println!();
    println!("player 2 strategies:");
    let sorted_items: Vec<_> = i_map.iter().filter(|(key, _)| key.len() % 2 == 1).collect();
    for (_, v) in sorted_items {
        println!("{}", v);
    }
}
