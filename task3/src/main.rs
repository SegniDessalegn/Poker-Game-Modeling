use lazy_static::lazy_static;
use rand::Rng;
use std::collections::HashMap;
use std::fmt;

const N_ACTIONS: usize = 3;
const N_CARDS: usize = 3;
const CHANCE_ACTIONS: [&str; N_ACTIONS] = ["c", "r", "f"]; // There are three actions, 'c' => call, 'r' => raise, 'f' => fold

lazy_static! {
    static ref PUBLIC_CARD: isize = rand::thread_rng().gen_range(0..=2); // Assuming card values range from 0 to 2
}

// Predefined terminal histories
const TERMINALS: &[&str] = &[
    "iicccc", "iicccrc", "iicccrf", "iicccf", "iiccrc", "iiccrrc", "iicccr", "iiccrrf", "iiccrf",
    "iiccf", "iicrcc", "iicrcrc", "iicrcrf", "iicrcf", "iicrrc", "iicrrf", "iicrf", "iicf",
    "iirccc", "iirccrc", "iirccrf", "iirccf", "iircrc", "iircrf", "iircf", "iirrcc", "iirrcf",
    "iirrcc", "iirrf", "iirf", "iif",
];

fn main() {
    let mut i_map: HashMap<String, InformationSet> = HashMap::new();
    let n_iterations = 10000;
    let mut expected_game_value = 0.0;

    for i in 0..n_iterations {
        expected_game_value += cfr(&mut i_map, "", -1, -1, 1.0, 1.0, 1.0, 0);

        for v in i_map.values_mut() {
            v.next_strategy();
        }
        println!(
            "iteration {}, expected game value: {}",
            i,
            expected_game_value / n_iterations as f64
        );
    }

    expected_game_value /= n_iterations as f64;
    display_results(expected_game_value, &i_map);
}

// The CFR (Counter Factual Regret Minimization) algorithm
pub fn cfr(
    i_map: &mut HashMap<String, InformationSet>,
    history: &str,
    card_1: isize,
    card_2: isize,
    pr_1: f64,
    pr_2: f64,
    pr_c: f64,
    bet: isize,
) -> f64 {
    if is_chance_node(history) {
        return chance_util(i_map);
    }

    if is_terminal(history) {
        return terminal_util(history, card_1, card_2) as f64;
    }

    let n = history.len();
    let is_player_1 = n % 2 == 0;

    let (key, mut info_set) =
        get_info_set(i_map, if is_player_1 { card_1 } else { card_2 }, history);

    let strategy = &info_set.strategy;

    info_set.reach_pr += if is_player_1 { pr_1 } else { pr_2 };

    let mut action_utils = [0.0; N_ACTIONS];

    for (i, action) in CHANCE_ACTIONS.iter().enumerate() {
        if action == &"r" && bet == 4 {
            continue;
        }

        let next_history = format!("{}{}", history, action);
        if is_player_1 {
            action_utils[i] = -1.0
                * cfr(
                    i_map,
                    &next_history,
                    card_1,
                    card_2,
                    pr_1 * strategy[i],
                    pr_2,
                    pr_c,
                    bet + if action == &"r" { 2 } else { 0 },
                );
        } else {
            action_utils[i] = -1.0
                * cfr(
                    i_map,
                    &next_history,
                    card_1,
                    card_2,
                    pr_1,
                    pr_2 * strategy[i],
                    pr_c,
                    bet + if action == &"r" { 2 } else { 0 },
                );
        }
    }

    let util = action_utils
        .iter()
        .zip(strategy.iter())
        .map(|(&x, &y)| x * y)
        .sum();
    let regrets: Vec<f64> = action_utils
        .iter()
        .zip(strategy.iter())
        .map(|(&_x, &_y)| _x - util)
        .collect();

    let (pr_1_factor, pr_2_factor) = if is_player_1 {
        (pr_2, pr_c)
    } else {
        (pr_1, pr_c)
    };

    info_set
        .regret_sum
        .iter_mut()
        .zip(regrets.iter())
        .for_each(|(a, &b)| *a += pr_1_factor * pr_2_factor * b);

    i_map.insert(key, info_set);

    util
}

// Determine if we are at the natural moves (random chance nodes)
fn is_chance_node(history: &str) -> bool {
    history.is_empty()
}

// Start the CFR algorithm by iterating through every combination of chance nodes
fn chance_util(i_map: &mut HashMap<String, InformationSet>) -> f64 {
    let mut expected_value = 0.0;
    let n_possibilities = 6;
    for i in 0..N_CARDS {
        for j in 0..N_CARDS {
            if i != j {
                expected_value += cfr(
                    i_map,
                    "ii",
                    i as isize,
                    j as isize,
                    1.0,
                    1.0,
                    1.0 / n_possibilities as f64,
                    0,
                );
            }
        }
    }
    expected_value / n_possibilities as f64
}

// Check if we have reached the terminal history
fn is_terminal(history: &str) -> bool {
    TERMINALS.contains(&history)
}

// Calculate the terminal utility
fn terminal_util(history: &str, card_1: isize, card_2: isize) -> f64 {
    let card_val = [("J", 1), ("Q", 2), ("K", 3)];

    let n = history.len();
    let card_player = if n % 2 == 0 { 1 } else { -1 };

    let net = get_pot(history, card_player) as f64;

    if history.ends_with('f') {
        return -((card_player as f64) * net);
    } else if card_player == 1 {
        if card_1 == *PUBLIC_CARD {
            return (card_player as f64) * net;
        } else if card_2 == *PUBLIC_CARD {
            return -((card_player as f64) * net);
        } else {
            let val_1 = card_val
                .iter()
                .find(|&&(s, _)| s == card_str(card_1 as usize))
                .unwrap()
                .1;
            let val_2 = card_val
                .iter()
                .find(|&&(s, _)| s == card_str(card_2 as usize))
                .unwrap()
                .1;
            return (card_player as f64) * (if val_1 > val_2 { 1.0 } else { -1.0 }) * net;
        }
    } else {
        if card_2 == *PUBLIC_CARD {
            return -((card_player as f64) * net);
        } else if card_1 == *PUBLIC_CARD {
            return (card_player as f64) * net;
        } else {
            let val_1 = card_val
                .iter()
                .find(|&&(s, _)| s == card_str(card_1 as usize))
                .unwrap()
                .1;
            let val_2 = card_val
                .iter()
                .find(|&&(s, _)| s == card_str(card_2 as usize))
                .unwrap()
                .1;
            return -((card_player as f64) * (if val_1 > val_2 { 1.0 } else { -1.0 }) * net);
        }
    }
}

// Calculate the amount of chipis in the pot based on the given game history
fn get_pot(history: &str, turn: isize) -> isize {
    let mut bet = [0, 0];
    let mut prev = 1;
    let mut player = 0;
    let pot = 1;
    let mut bet_round = 1;

    for &op in history.as_bytes().iter().skip(2) {
        match op as char {
            'r' => {
                bet[player] += prev - ((bet_round * 2) - bet[player]);
                prev += 2;
                bet_round += 1;
            }
            'c' => {
                bet[player] += prev - bet[player];
            }
            _ => {}
        }

        player = (player + 1) % 2;
    }

    pot + if turn == -1 { bet[0] } else { bet[1] }
}

// Retrieve information set from dictionary
fn get_info_set(
    i_map: &mut HashMap<String, InformationSet>,
    card: isize,
    history: &str,
) -> (String, InformationSet) {
    let key = format!("{} {}", card_str(card as usize), history);

    let info_set = i_map
        .remove(&key)
        .unwrap_or_else(|| InformationSet::new(&key));

    (key, info_set)
}

fn card_str(card: usize) -> &'static str {
    let combs = ["J", "Q", "K"];
    return combs[card];
}

#[derive(Debug)]
pub struct InformationSet {
    pub key: String,
    pub regret_sum: [f64; N_ACTIONS],
    pub strategy_sum: [f64; N_ACTIONS],
    pub strategy: [f64; N_ACTIONS],
    pub reach_pr: f64,
    pub reach_pr_sum: f64,
}

impl InformationSet {
    pub fn new(key: &str) -> InformationSet {
        InformationSet {
            key: key.to_string(),
            regret_sum: [0.0; N_ACTIONS],
            strategy_sum: [0.0; N_ACTIONS],
            strategy: [1.0 / N_ACTIONS as f64; N_ACTIONS],
            reach_pr: 0.0,
            reach_pr_sum: 0.0,
        }
    }

    pub fn next_strategy(&mut self) {
        self.strategy_sum
            .iter_mut()
            .zip(self.strategy.iter())
            .for_each(|(a, &b)| *a += self.reach_pr * b);

        self.strategy = self.calc_strategy();
        self.reach_pr_sum += self.reach_pr;
        self.reach_pr = 0.0;
    }

    fn calc_strategy(&self) -> [f64; N_ACTIONS] {
        let strategy = self.make_positive(self.regret_sum);

        let total = strategy.iter().sum::<f64>();

        if total > 0.0 {
            strategy.map(|x| x / total)
        } else {
            [1.0 / N_ACTIONS as f64; N_ACTIONS]
        }
    }

    fn get_average_strategy(&self) -> [f64; N_ACTIONS] {
        let strategy = self.strategy_sum.map(|x| x / self.reach_pr_sum);

        let strategy = strategy.map(|x| if x < 0.001 { 0.0 } else { x });

        let total = strategy.iter().sum::<f64>();

        if total > 0.0 {
            strategy.map(|x| x / total)
        } else {
            [1.0 / N_ACTIONS as f64; N_ACTIONS]
        }
    }

    fn make_positive(&self, x: [f64; N_ACTIONS]) -> [f64; N_ACTIONS] {
        x.map(|val| val.max(0.0))
    }
}

impl fmt::Display for InformationSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let strategies: Vec<String> = self
            .get_average_strategy()
            .iter()
            .map(|x| format!("{:03.2}", x))
            .collect();

        let mut card = self.key.clone();
        if card.split_whitespace().last().map_or(0, |s| s.len()) > 3 {
            card = format!(
                "{}{}{}",
                &card[0..1],
                card_str(*PUBLIC_CARD as usize),
                &card[1..]
            );
        }

        write!(f, "{} [{}]", card, strategies.join(", "))
    }
}

fn display_results(ev: f64, i_map: &HashMap<String, InformationSet>) {
    println!();
    println!("==== notation ====");
    println!("i => initial");
    println!("==================");
    println!();

    println!("player 1 expected value: {}", ev);
    println!("player 2 expected value: {}", -1.0 * ev);
    println!();

    println!("******* PUBLIC CARD ********");
    println!(
        "-----------> {} <------------",
        card_str(*PUBLIC_CARD as usize)
    );
    println!("****************************");
    println!();

    let mut items = i_map.iter().collect::<Vec<_>>();

    items.sort_by(|a, b| a.0.cmp(b.0));

    let (p1_items, p2_items) = items
        .into_iter()
        .partition::<Vec<_>, _>(|(k, _)| k.len() % 2 == 0);

    println!("\nplayer 1 strategies:");
    for (_, v) in p1_items {
        println!("{}", v);
    }

    println!("\nplayer 2 strategies:");
    for (_, v) in p2_items {
        println!("{}", v);
    }
}
