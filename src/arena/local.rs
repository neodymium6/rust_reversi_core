use crate::arena::core::{Arena, Player};
use crate::arena::error::ArenaError;
use crate::board::Turn;
use std::process::Stdio;
use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command},
};

#[derive(Debug)]
pub struct LocalArena {
    command1: Vec<String>,
    command2: Vec<String>,
    stats: (usize, usize, usize),
    pieces: (usize, usize),
    show_progress: bool,
}

type ProcessPlayer = Player<ChildStdin, BufReader<ChildStdout>>;
type ProcessTuple = (Child, ChildStdin, BufReader<ChildStdout>);
type ProcessResult = Result<(ProcessTuple, ProcessTuple), ArenaError>;
type ProcessPair = (Child, Child);
type PlayerPair = (ProcessPlayer, ProcessPlayer);
impl LocalArena {
    /// Create a new LocalArena instance
    /// # Arguments
    /// * `command1` - Command to start the first player
    /// * `command2` - Command to start the second player
    /// * `show_progress` - Show progress bar
    /// # Returns
    /// * `LocalArena` instance
    /// # Example
    /// ```
    /// use rust_reversi_core::arena::LocalArena;
    /// let command1 = vec!["./player1".to_string()];
    /// let command2 = vec!["./player2".to_string()];
    /// let show_progress = true;
    /// let arena = LocalArena::new(command1, command2, show_progress);
    /// ```
    /// # Note
    /// * The command should be a vector of strings
    /// * The first element of the vector should be the path to the executable
    /// * The rest of the elements are arguments to the executable
    /// * The player should print "pong" after receiving "ping" to confirm the connection
    pub fn new(command1: Vec<String>, command2: Vec<String>, show_progress: bool) -> Self {
        LocalArena {
            command1,
            command2,
            stats: (0, 0, 0),
            pieces: (0, 0),
            show_progress,
        }
    }

    fn start_process(
        command: &[String],
        turn: Turn,
    ) -> Result<(Child, ChildStdin, BufReader<ChildStdout>), std::io::Error> {
        let mut cmd = Command::new(&command[0]);
        for arg in command.iter().skip(1) {
            cmd.arg(arg);
        }

        match turn {
            Turn::Black => cmd.arg("BLACK"),
            Turn::White => cmd.arg("WHITE"),
        };

        let mut process = cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;

        let mut stdin = process.stdin.take().unwrap();
        let stdout = process.stdout.take().unwrap();

        // ping-pong test
        writeln!(stdin, "ping")
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Write error"))?;
        stdin
            .flush()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Flush error"))?;

        let mut reader = BufReader::new(stdout);
        let mut response = String::new();
        reader
            .read_line(&mut response)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Read error"))?;

        if response.trim() != "pong" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid response",
            ));
        }

        Ok((process, stdin, reader))
    }

    fn init_processes(&self, p1_turn: Turn) -> ProcessResult {
        let (mut process1, stdin1, stdout1) = Self::start_process(&self.command1, p1_turn)
            .map_err(|_| ArenaError::EngineStartError)?;

        let p2_turn = p1_turn.opposite();
        if let Ok((process2, stdin2, stdout2)) = Self::start_process(&self.command2, p2_turn) {
            Ok(((process1, stdin1, stdout1), (process2, stdin2, stdout2)))
        } else {
            process1.kill().map_err(|_| ArenaError::EngineEndError)?;
            process1.wait().map_err(|_| ArenaError::EngineEndError)?;
            Err(ArenaError::EngineStartError)
        }
    }

    fn get_players(&mut self) -> Result<(Vec<ProcessPair>, Vec<PlayerPair>), ArenaError> {
        // P1equalsBlack
        let ((process1, stdin1, stdout1), (process2, stdin2, stdout2)) =
            self.init_processes(Turn::Black)?;
        let player_1b = Player::new(stdin1, stdout1);
        let player_2w = Player::new(stdin2, stdout2);

        // P2equalsBlack
        let ((process3, stdin3, stdout3), (process4, stdin4, stdout4)) =
            self.init_processes(Turn::White)?;
        let player_2b = Player::new(stdin4, stdout4);
        let player_1w = Player::new(stdin3, stdout3);

        Ok((
            vec![(process1, process2), (process3, process4)],
            vec![(player_1b, player_2w), (player_2b, player_1w)],
        ))
    }

    /// Play n games between the two players
    /// # Arguments
    /// * `n` - Number of games to play
    /// # Returns
    /// * `Result<(), ArenaError>` - Ok(()) if successful, Err(ArenaError) otherwise
    /// # Note
    /// * n should be a positive even number
    pub fn play_n(&mut self, n: usize) -> Result<(), ArenaError> {
        let (mut processes, players) = self.get_players()?;

        let mut arena = Arena::new(players, self.show_progress);
        if let Err(e) = arena.play_n(n) {
            for (p1, p2) in processes.iter_mut() {
                p1.kill().map_err(|_| ArenaError::EngineEndError)?;
                p1.wait().map_err(|_| ArenaError::EngineEndError)?;
                p2.kill().map_err(|_| ArenaError::EngineEndError)?;
                p2.wait().map_err(|_| ArenaError::EngineEndError)?;
            }
            return Err(e);
        }
        let (p1_win, p2_win, draw) = arena.get_stats();
        self.stats.0 += p1_win;
        self.stats.1 += p2_win;
        self.stats.2 += draw;
        let (p1_pieces, p2_pieces) = arena.get_pieces();
        self.pieces.0 += p1_pieces;
        self.pieces.1 += p2_pieces;

        // drop all processes
        for (p1, p2) in processes.iter_mut() {
            p1.kill().map_err(|_| ArenaError::EngineEndError)?;
            p1.wait().map_err(|_| ArenaError::EngineEndError)?;
            p2.kill().map_err(|_| ArenaError::EngineEndError)?;
            p2.wait().map_err(|_| ArenaError::EngineEndError)?;
        }

        Ok(())
    }

    /// Get the statistics of the games played
    /// # Returns
    /// * `(usize, usize, usize)` - Number of wins for player 1, player 2 and draws
    /// # Note
    /// * stats are cumulative
    /// * stats are not reset after each call to play_n
    /// * stats are reset after each call to new
    pub fn get_stats(&self) -> (usize, usize, usize) {
        self.stats
    }

    /// Get the number of pieces played in the games
    /// # Returns
    /// * `(usize, usize)` - Number of pieces played by player 1 and player 2
    /// # Note
    /// * pieces are cumulative
    /// * pieces are not reset after each call to play_n
    /// * pieces are reset after each call to new
    pub fn get_pieces(&self) -> (usize, usize) {
        self.pieces
    }
}
