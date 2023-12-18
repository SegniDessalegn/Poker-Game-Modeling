use std::collections::HashSet;

struct Game {
    initial_pot: i32,
    operations: std::collections::HashMap<&'static str, Vec<&'static str>>,
}

impl Game {
    fn new(initial_pot: i32) -> Game {
        let mut operations = std::collections::HashMap::new();
        operations.insert("x", vec!["x", "b"]);
        operations.insert("b", vec!["r", "c", "f"]);
        operations.insert("c", vec!["c", "f", "r"]);
        operations.insert("f", vec!["r", "c", "f"]);
        operations.insert("r", vec!["c", "f"]);

        Game {
            initial_pot,
            operations,
        }
    }

    fn calculate(&self, seq: &[&str]) -> i32 {
        let mut bet = [0, 0, 0];
        let mut prev = 0;
        let mut player = 0;

        for &op in seq {
            match op {
                "b" => {
                    prev = 10;
                    bet[player] += 10 - bet[player];
                }
                "r" => {
                    prev = 20;
                    bet[player] += 20 - bet[player];
                }
                "c" => {
                    bet[player] += prev - bet[player];
                }
                _ => (),
            }

            player = (player + 1) % 3;
        }

        self.initial_pot + bet.iter().sum::<i32>()
    }

    fn choose_next_player(&self, curr_player: usize, fold: &HashSet<usize>) -> usize {
        let mut next_player = curr_player + 1;
        next_player %= 3;

        while fold.contains(&next_player) {
            next_player += 1;
            next_player %= 3;
        }

        next_player
    }

    fn generate_seq(
        &self,
        curr_player: usize,
        prev_op: &'static str,
        player_stop: usize,
        seq: &mut Vec<&'static str>,
        fold: &mut HashSet<usize>,
        raised: bool,
    ) {
        if curr_player == player_stop || fold.len() == 2 {
            println!(
                "{}",
                seq.iter()
                    .map(|&s| match s {
                        "b" => "b10",
                        "r" => "r20",
                        _ => s,
                    })
                    .collect::<Vec<&str>>()
                    .join(":")
                    + ", pot="
                    + &self.calculate(&seq).to_string()
            );
            return;
        }

        let next_player = self.choose_next_player(curr_player, &fold);

        for &next_op in &self.operations[prev_op] {
            if next_op == "r" && raised {
                continue;
            }

            if next_op == "f" {
                fold.insert(curr_player);
            }

            let mut next_player_stop = player_stop;
            if next_op == "r" || next_op == "b" {
                next_player_stop = curr_player;
            }

            let new_raised = if next_op == "r" { true } else { raised };

            seq.push(next_op);

            self.generate_seq(
                next_player,
                next_op,
                next_player_stop,
                seq,
                fold,
                new_raised,
            );

            seq.pop();
            if next_op == "f" {
                fold.remove(&curr_player);
            }
        }
    }

    fn generate(&self) {
        let mut seq1 = vec!["x"];
        let mut fold1 = HashSet::new();
        self.generate_seq(1, "x", 0, &mut seq1, &mut fold1, false);

        let mut seq2 = vec!["b"];
        let mut fold2 = HashSet::new();
        self.generate_seq(1, "b", 0, &mut seq2, &mut fold2, false);
    }
}

fn main() {
    let game = Game::new(60);
    game.generate();
}
