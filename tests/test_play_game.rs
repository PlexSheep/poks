use poks::game::{GameSetup, World};

use ntest::timeout;

fn get_world() -> World {
    World::new(8, GameSetup::CPUOnly)
}

#[test]
#[timeout(3)]
fn test_play_50_games_cpu() {
    let mut w = get_world();
    for _gi in 0..50 {
        w.start_new_game();
        while !w.game.is_finished() {
            w.tick_game().unwrap();
            let last_action = w.action_log().iter().last().unwrap();
            if let Some(pid) = last_action.0 {
                println!("Player {pid}: {}", last_action.1)
            } else {
                println!("{}", last_action.1)
            }
        }
    }
}
