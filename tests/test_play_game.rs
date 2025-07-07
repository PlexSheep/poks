use ntest::timeout;
use poks::{CU, lobby::Lobby, player::PlayerCPU};

fn get_world() -> Lobby {
    let mut wb = Lobby::builder();
    for _ in 0..8 {
        wb.add_player(Box::new(PlayerCPU::default())).unwrap();
    }
    for player in wb.players.iter_mut() {
        player.set_currency(CU!(5000));
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
