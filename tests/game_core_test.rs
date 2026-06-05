#[cfg(test)]
mod tests {
    use snake::{GameCore, GameState, MyVec2, PlayerState};

    const ALICE: &str = "Alice";
    const BOB: &str = "Bob";

    #[test]
    fn new_game_is_not_started() {
        let game = GameCore::new(false);

        assert!(matches!(game.state, GameState::NotStarted));
        assert!(game.players.is_empty());
        assert!(game.fruit_pos.is_none());
    }

    #[test]
    fn add_player_adds_not_ready_player() {
        let mut game = GameCore::new(false);

        game.add_player(ALICE);

        assert_eq!(game.players.len(), 1);

        let player = game.players.get(ALICE).unwrap();
        assert_eq!(player.name, ALICE);
        assert_eq!(player.state, PlayerState::NotReady);
    }

    #[test]
    fn remove_player_removes_player_and_resets_game() {
        let mut game = GameCore::new(false);

        game.add_player(ALICE);
        game.add_player(BOB);

        game.state = GameState::Playing;
        game.fruit_pos = Some(MyVec2::new(10.0, 10.0));

        game.remove_player(ALICE);

        assert_eq!(game.players.len(), 1);
        assert!(!game.players.contains_key(ALICE));

        assert!(matches!(game.state, GameState::NotStarted));
        assert!(game.fruit_pos.is_none());
    }

    #[test]
    fn reset_game_state_clears_runtime_state() {
        let mut game = GameCore::new(false);

        game.state = GameState::Paused;
        game.fruit_pos = Some(MyVec2::new(20.0, 20.0));

        game.reset_game_state();

        assert!(matches!(game.state, GameState::NotStarted));
        assert!(game.fruit_pos.is_none());
    }

    #[test]
    fn enter_sets_player_ready() {
        let mut game = GameCore::new(false);

        game.add_player(ALICE);

        game.handle_input(ALICE, '\r');

        assert_eq!(game.players.get(ALICE).unwrap().state, PlayerState::Ready);
    }

    #[test]
    fn escape_sets_player_not_ready() {
        let mut game = GameCore::new(false);

        game.add_player(ALICE);

        game.handle_input(ALICE, '\r');
        game.handle_input(ALICE, '\x1B');

        assert_eq!(
            game.players.get(ALICE).unwrap().state,
            PlayerState::NotReady
        );
    }

    #[test]
    fn finish_game_with_winner() {
        let mut game = GameCore::new(false);

        game.add_player(ALICE);
        game.add_player(BOB);

        game.players.get_mut(ALICE).unwrap().state = PlayerState::Ready;
        game.players.get_mut(BOB).unwrap().state = PlayerState::Ready;

        game.finish_the_game(Some(ALICE));

        match &game.state {
            GameState::Finished(details) => {
                assert!(!details.draw);
                assert_eq!(details.winner, ALICE);
            }
            _ => panic!("Game should be finished"),
        }

        assert_eq!(
            game.players.get(ALICE).unwrap().state,
            PlayerState::NotReady
        );

        assert_eq!(
            game.players.get(BOB).unwrap().state,
            PlayerState::NotReady
        );
    }

    #[test]
    fn finish_game_as_draw() {
        let mut game = GameCore::new(false);

        game.finish_the_game(None);

        match &game.state {
            GameState::Finished(details) => {
                assert!(details.draw);
                assert_eq!(details.winner, "");
            }
            _ => panic!("Game should be finished"),
        }
    }

    #[test]
    fn get_enemy_name_returns_other_player() {
        let mut game = GameCore::new(false);

        game.add_player(ALICE);
        game.add_player(BOB);

        let enemy = game.get_enemy_name(ALICE);

        assert_eq!(enemy, BOB);
    }

    #[test]
    fn update_starts_game_when_all_players_ready() {
        let mut game = GameCore::new(false);

        game.add_player(ALICE);
        game.add_player(BOB);

        game.players.get_mut(ALICE).unwrap().state = PlayerState::Ready;
        game.players.get_mut(BOB).unwrap().state = PlayerState::Ready;

        game.update();

        assert!(matches!(game.state, GameState::Playing));
    }
}
