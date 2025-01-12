use rust_reversi_core::arena::LocalArena;
use rust_reversi_core::arena::*;
use rust_reversi_core::arena::{NetworkArenaClient, NetworkArenaServer};
use std::thread;
use std::time::Duration;

mod players;

const N_GAMES: usize = 1000;
const TEST_PORT: u16 = 12345;
const TEST_PORT2: u16 = 12346;

#[cfg(test)]
mod tests {
    use super::*;
    use players::compile_player;
    use players::get_player_path;

    #[test]
    fn random_vs_random() {
        compile_player("random_player");
        let random_player = get_player_path("random_player");

        let command1 = vec![random_player.to_str().unwrap().to_string()];
        let command2 = command1.clone();

        let mut arena = LocalArena::new(command1, command2, false);
        arena.play_n(N_GAMES).unwrap();

        let (wins1, wins2, draws) = arena.get_stats();
        let (pieces1, pieces2) = arena.get_pieces();

        assert_eq!(wins1 + wins2 + draws, N_GAMES);
        assert!(pieces1 + pieces2 > 0);

        let win_ratio = (wins1 as f64 - wins2 as f64).abs() / N_GAMES as f64;
        assert!(
            win_ratio < 0.1,
            "Win ratio is too unbalanced: {}",
            win_ratio
        );
    }

    #[test]
    fn arena_odd_games() {
        compile_player("random_player");
        let random_player = get_player_path("random_player");

        let command1 = vec![random_player.to_str().unwrap().to_string()];
        let command2 = command1.clone();

        let mut arena = LocalArena::new(command1, command2, false);
        let result = arena.play_n(999);
        assert!(matches!(result, Err(ArenaError::GameNumberInvalid)));
    }

    #[test]
    fn arena_invalid_player() {
        compile_player("random_player");
        let invalid_player = "nonexistent_player".to_string();
        let ranmdom_player = get_player_path("random_player");
        let command1 = vec![ranmdom_player.to_str().unwrap().to_string()];
        let command2 = vec![invalid_player];

        let mut arena = LocalArena::new(command1, command2, false);
        let result = arena.play_n(2);
        assert!(matches!(result, Err(ArenaError::EngineStartError)));
    }

    #[test]
    fn arena_multiple_sessions() {
        compile_player("random_player");
        let random_player = get_player_path("random_player");

        let command1 = vec![random_player.to_str().unwrap().to_string()];
        let command2 = command1.clone();

        let mut arena = LocalArena::new(command1, command2, false);

        // First session
        arena.play_n(100).unwrap();
        let (wins1, wins2, draws) = arena.get_stats();
        assert_eq!(wins1 + wins2 + draws, 100);

        // Second session
        arena.play_n(100).unwrap();
        let (wins1, wins2, draws) = arena.get_stats();
        assert_eq!(wins1 + wins2 + draws, 200);
    }

    #[test]
    fn arena_timeout() {
        compile_player("slow_player");
        compile_player("random_player");

        let slow_player = get_player_path("slow_player");
        let random_player = get_player_path("random_player");

        let command1 = vec![slow_player.to_str().unwrap().to_string()];
        let command2 = vec![random_player.to_str().unwrap().to_string()];

        let mut arena = LocalArena::new(command1, command2, false);
        let result = arena.play_n(2);
        assert!(matches!(result, Err(ArenaError::GameError(_))));
    }

    #[test]
    fn network_arena_basic() {
        compile_player("random_player");
        let random_player = get_player_path("random_player");

        let command = vec![random_player.to_str().unwrap().to_string()];

        // Start server
        let mut server = NetworkArenaServer::new(N_GAMES, false).unwrap();
        thread::spawn(move || {
            server.start("127.0.0.1".to_string(), TEST_PORT).unwrap();
        });

        thread::sleep(Duration::from_millis(100)); // Wait for server to start

        // Start clients
        let mut client1 = NetworkArenaClient::new(command.clone());
        let mut client2 = NetworkArenaClient::new(command);

        let client1_handle = thread::spawn(move || {
            client1.connect("127.0.0.1".to_string(), TEST_PORT).unwrap();
            client1
        });

        let client2_handle = thread::spawn(move || {
            client2.connect("127.0.0.1".to_string(), TEST_PORT).unwrap();
            client2
        });

        let client1 = client1_handle.join().unwrap();
        client2_handle.join().unwrap();

        let (wins1, losses1, draws1) = client1.get_stats();
        let (pieces1, opponent_pieces1) = client1.get_pieces();

        assert_eq!(wins1 + losses1 + draws1, N_GAMES);
        assert!(pieces1 + opponent_pieces1 > 0);

        let win_ratio = (wins1 as f64 - losses1 as f64).abs() / N_GAMES as f64;
        assert!(
            win_ratio < 0.1,
            "Win ratio is too unbalanced: {}",
            win_ratio
        );
    }

    #[test]
    fn network_arena_invalid_game_count() {
        let result = NetworkArenaServer::new(99, false);
        assert!(matches!(
            result,
            Err(NetworkArenaServerError::GameNumberInvalid)
        ));
    }

    #[test]
    fn network_arena_invalid_player() {
        let invalid_player = vec!["nonexistent_player".to_string()];

        let mut server = NetworkArenaServer::new(N_GAMES, false).unwrap();
        thread::spawn(move || {
            server.start("127.0.0.1".to_string(), TEST_PORT2).unwrap();
        });

        thread::sleep(Duration::from_millis(100));

        let mut client = NetworkArenaClient::new(invalid_player);
        let result = client.connect("127.0.0.1".to_string(), TEST_PORT);
        assert!(result.is_err());
    }
}
