use ntest::timeout;
use poks::{player::PlayerCPU, world::World};

fn get_world() -> World {
    let mut wb = World::builder();
    for _ in 0..8 {
        wb.add_player(Box::new(PlayerCPU::default())).unwrap();
    }
    wb.build().unwrap()
}

#[test]
#[timeout(300)]
fn test_play_50_games_cpu() {
    let mut w = get_world();
    for _gi in 0..50 {
        w.start_new_game().unwrap();
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
