use std::collections::HashMap;
use std::fmt;

const N_ACTIONS: usize = 5;
const N_CARDS: usize = 3;

pub fn cfr(
    i_map: &mut HashMap<String, InformationSet>,
    history: &str,
    card_1: isize,
    card_2: isize,
    pr_1: f64,
    pr_2: f64,
    pr_c: f64,
) -> f64 {
    if history.is_empty() {
        return chance_util(i_map);
    }

    if history.len() == 4 {
        return terminal_util(history, card_1, card_2);
    }

    let n = history.len();
    let is_player_1 = n % 2 == 0;

    let (key, mut info_set) = get_info_set(i_map, if is_player_1 { card_1 } else { card_2 }, history);

    let strategy = &info_set.strategy;
    info_set.reach_pr += if is_player_1 { pr_1 } else { pr_2 };

    let mut action_utils = [0.0; N_ACTIONS];

    for (i, action) in ["c", "b", "f"].iter().enumerate() {
        let next_history = format!("{}{}", history, action);
        let new_pr_1 = if is_player_1 { pr_1 * strategy[i] } else { pr_1 };
        let new_pr_2 = if is_player_1 { pr_2 } else { pr_2 * strategy[i] };
        action_utils[i] = -1.0 * cfr(i_map, &next_history, card_1, card_2, new_pr_1, new_pr_2, pr_c);
    }

    let util = action_utils.iter().zip(strategy.iter()).map(|(&x, &y)| x * y).sum();
    let regrets: Vec<f64> = action_utils.iter().zip(strategy.iter()).map(|(&_x, &_y)| _x - util).collect();

    let (pr_1_factor, pr_2_factor) = if is_player_1 { (pr_2, pr_c) } else { (pr_1, pr_c) };

    info_set.regret_sum.iter_mut().zip(regrets.iter()).for_each(|(a, &b)| *a += pr_1_factor * pr_2_factor * b);

    i_map.insert(key, info_set);

    util
}

fn chance_util(i_map: &mut HashMap<String, InformationSet>) -> f64 {
    let mut expected_value = 0.0;
    let n_possibilities = 6;
    for i in 0..N_CARDS {
        for j in 0..N_CARDS {
            if i != j {
                expected_value += cfr(
                    i_map,
                    "r",
                    i as isize,
                    j as isize,
                    1.0,
                    1.0,
                    1.0 / n_possibilities as f64,
                );
            }
        }
    }
    expected_value / n_possibilities as f64
}

fn terminal_util(history: &str, card_1: isize, card_2: isize) -> f64 {
    let card_player = if history.chars().next().unwrap() == 'r' { card_1 } else { card_2 };
    let card_opponent = if history.chars().next().unwrap() == 'r' { card_2 } else { card_1 };
    match &history[2..] {
        "cc" => if card_player > card_opponent { 1.0 } else { -1.0 },
        "bb" => if card_player > card_opponent { 2.0 } else { -2.0 },
        "bc" | "cb" => 1.0,
        _ => 0.0,
    }
}

fn get_info_set(i_map: &mut HashMap<String, InformationSet>, card: isize, history: &str) -> (String, InformationSet) {
    let key = format!("{} {}", card_str(card as usize), history);
    let info_set = i_map.remove(&key).unwrap_or_else(|| InformationSet::new(&key));
    (key, info_set)
}

fn card_str(card: usize) -> &'static str {
    match card {
        0 => "J",
        1 => "Q",
        _ => "K",
    }
}

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
        self.strategy_sum.iter_mut().zip(self.strategy.iter()).for_each(|(a, &b)| *a += self.reach_pr * b);
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
        write!(f, "{} {:?}", self.key, self.get_average_strategy())
    }
}

fn main() {
    let mut i_map: HashMap<String, InformationSet> = HashMap::new();
    let n_iterations = 20000;
    let mut expected_game_value = 0.0;

    for _ in 0..n_iterations {
        expected_game_value += cfr(&mut i_map, "", -1, -1, 1.0, 1.0, 1.0);
        for v in i_map.values_mut() {
            v.next_strategy();
        }
    }

    expected_game_value /= n_iterations as f64;
    display_results(expected_game_value, &i_map);
}

fn display_results(ev: f64, i_map: &HashMap<String, InformationSet>) {
    println!("player 1 expected value: {}", ev);
    println!("player 2 expected value: {}", -1.0 * ev);

    let mut items = i_map.iter().collect::<Vec<_>>();
    items.sort_by(|a, b| a.0.cmp(b.0));

    let (p1_items, p2_items) = items.into_iter().partition::<Vec<_>, _>(|(k, _)| k.len() % 2 == 0);

    println!("\nplayer 1 strategies:");
    for (_, v) in p1_items {
        println!("{}", v);
    }

    println!("\nplayer 2 strategies:");
    for (_, v) in p2_items {
        println!("{}", v);
    }
}
