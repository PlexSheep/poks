use poks::game::World;

use ntest::timeout;

fn get_world() -> World {
    World::new(8)
}

#[test]
#[timeout(300)]
fn test_play_50_games_cpu() {
    let mut w = get_world();
    for _gi in 0..50 {
        w.start_new_game();
        while !w.game.is_finished() {
            w.tick_game().unwrap();
        }
    }
}
