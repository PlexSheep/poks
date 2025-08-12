#[allow(unused)] // i wanna keep that
use ntest::timeout;
use poksen::{
    CU, PoksError,
    lobby::Lobby,
    players::{PlayerCPU, Seat},
};

fn get_world() -> Lobby {
    let startc = CU!(5000);
    let mut wb = Lobby::builder();
    for _ in 0..8 {
        let seat = Seat::new(startc, PlayerCPU::default());
        wb.add_seat(seat).unwrap();
    }
    for player in wb.players.iter_mut() {
        player.set_currency(startc);
    }
    wb.build().unwrap()
}

#[test]
fn test_play_500_games_cpu() {
    let mut w = get_world();
    for _gi in 0..500 {
        w.start_new_game().unwrap();
        while !w.game.is_finished() {
            match w.tick_game() {
                Ok(_) => (),
                Err(e) => match e {
                    PoksError::RaiseNotAllowed => (),
                    _ => panic!("Error while ticking the game: {e}"),
                },
            }
        }
    }
}
