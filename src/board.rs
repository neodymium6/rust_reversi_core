use crate::utils::StackVec64;
use crate::v8di::LRMASK;
use core::fmt;
use std::arch::x86_64::*;
use std::hash::Hash;
use std::mem::swap;

const BOARD_SIZE: usize = 8;
const LINE_CHAR_BLACK: char = 'X';
const LINE_CHAR_WHITE: char = 'O';
const LINE_CHAR_EMPTY: char = '-';

#[derive(Debug)]
pub enum BoardError {
    InvalidPosition,
    InvalidMove,
    InvalidPass,
    InvalidState,
    GameNotOverYet,
    InvalidCharactor,
    NoLegalMove,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Turn {
    Black,
    White,
}

impl Turn {
    #[inline]
    /// Get the opposite turn
    /// # Example
    /// ```
    /// use rust_reversi_core::board::Turn;
    /// let turn = Turn::Black;
    /// assert_eq!(turn.opposite(), Turn::White);
    /// ```
    pub fn opposite(&self) -> Turn {
        match self {
            Turn::Black => Turn::White,
            Turn::White => Turn::Black,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    Empty,
    Black,
    White,
}

impl Color {
    fn opposite(&self) -> Color {
        match self {
            Color::Empty => Color::Empty,
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Board {
    player_board: u64,
    opponent_board: u64,
    turn: Turn,
    legal_moves_cache: Option<u64>,
}

const BITS: [u64; 64] = {
    let mut bits = [0u64; 64];
    let mut i = 0;
    while i < 64 {
        bits[i] = 1u64 << (63 - i);
        i += 1;
    }
    bits
};

impl Default for Board {
    fn default() -> Self {
        Board {
            player_board: 0x00_00_00_08_10_00_00_00,
            opponent_board: 0x00_00_00_10_08_00_00_00,
            turn: Turn::Black,
            legal_moves_cache: None,
        }
    }
}

impl Board {
    /// Create a new Board instance
    /// # Returns
    /// * `Board` instance
    /// # Note
    /// * The initial board state is as follows:
    /// ```
    /// use rust_reversi_core::board::Board;
    /// let board = Board::new();
    /// assert_eq!(board.to_string().unwrap(), format!(
    ///  "{}{}{}{}{}{}{}{}{}{}",
    ///   " |abcdefgh\n",
    ///   "-+--------\n",
    ///   "1|--------\n",
    ///   "2|--------\n",
    ///   "3|--------\n",
    ///   "4|---OX---\n",
    ///   "5|---XO---\n",
    ///   "6|--------\n",
    ///   "7|--------\n",
    ///   "8|--------\n",
    /// ).as_str());
    /// ```
    /// * X: Black, O: White
    /// * Black goes first
    pub fn new() -> Board {
        Board::default()
    }

    /// Get the current board state
    /// # Returns
    /// * Tuple of (player_board, opponent_board, turn)
    /// # Example
    /// ```
    /// use rust_reversi_core::board::Board;
    /// let board = Board::new();
    /// let (player_board, opponent_board, turn) = board.get_board();
    /// ```
    /// # Note
    /// * player_board: Bitboard of the player's stones
    /// * opponent_board: Bitboard of the opponent's stones
    /// * turn: Turn of the player
    pub fn get_board(&self) -> (u64, u64, Turn) {
        (self.player_board, self.opponent_board, self.turn)
    }

    /// Get the current turn
    /// # Returns
    /// * Turn of the player
    pub fn get_turn(&self) -> Turn {
        self.turn
    }

    /// Set the current board state
    /// # Arguments
    /// * `player_board` - Bitboard of the player's stones
    /// * `opponent_board` - Bitboard of the opponent's stones
    /// * `turn` - Turn of the player
    pub fn set_board(&mut self, player_board: u64, opponent_board: u64, turn: Turn) {
        self.player_board = player_board;
        self.opponent_board = opponent_board;
        self.turn = turn;
        self.legal_moves_cache = None;
    }

    /// Set the current board state from a string
    /// # Arguments
    /// * `board_str` - String representation of the board
    /// * `turn` - Turn of the player
    /// # Returns
    /// * `Result<(), BoardError>` - Ok(()) if successful, Err(BoardError) otherwise
    /// # Example
    /// ```
    /// use rust_reversi_core::board::{Board, Turn};
    /// let mut board = Board::new();
    /// board.set_board_str(
    ///   format!(
    ///     "{}{}{}{}{}{}{}{}",
    ///     "--------",
    ///     "--------",
    ///     "--OX----",
    ///     "---XO---",
    ///     "--OOO---",
    ///     "--OO----",
    ///     "---O----",
    ///     "---X----",
    ///   ).as_str(),
    ///   Turn::Black,
    /// ).unwrap();
    /// ```
    pub fn set_board_str(&mut self, board_str: &str, turn: Turn) -> Result<(), BoardError> {
        let mut black_board = 0;
        let mut white_board = 0;
        for (i, c) in board_str.chars().enumerate() {
            match c {
                LINE_CHAR_BLACK => black_board |= BITS[i],
                LINE_CHAR_WHITE => white_board |= BITS[i],
                LINE_CHAR_EMPTY => (),
                _ => {
                    return Err(BoardError::InvalidCharactor);
                }
            }
        }
        match turn {
            Turn::Black => self.set_board(black_board, white_board, Turn::Black),
            Turn::White => self.set_board(white_board, black_board, Turn::White),
        }
        self.legal_moves_cache = None;
        Ok(())
    }

    /// Get the current board state as a string
    /// # Returns
    /// * String representation of the board
    /// # Note
    /// * X: Black, O: White
    /// * format is same as `set_board_str`
    pub fn get_board_line(&self) -> Result<String, BoardError> {
        let mut board_str = String::new();
        let player_char = match self.turn {
            Turn::Black => LINE_CHAR_BLACK,
            Turn::White => LINE_CHAR_WHITE,
        };
        let opponent_char = match self.turn {
            Turn::Black => LINE_CHAR_WHITE,
            Turn::White => LINE_CHAR_BLACK,
        };
        for &pos in BITS.iter() {
            match (self.player_board & pos, self.opponent_board & pos) {
                (0, 0) => board_str.push(LINE_CHAR_EMPTY),
                (_, 0) => board_str.push(player_char),
                (0, _) => board_str.push(opponent_char),
                (_, _) => return Err(BoardError::InvalidState),
            }
        }
        Ok(board_str)
    }

    /// Get the current board state as a vector of colors
    /// # Returns
    /// * Vector of colors
    /// # Note
    /// * Color::Black: player's stone
    /// * Color::White: opponent's stone
    /// * Color::Empty: empty
    pub fn get_board_vec_black(&self) -> Result<Vec<Color>, BoardError> {
        let mut board_vec = vec![Color::Empty; BOARD_SIZE * BOARD_SIZE];
        for (i, board_vec_elem) in board_vec.iter_mut().enumerate() {
            let bit = BITS[i];
            *board_vec_elem = match (self.player_board & bit, self.opponent_board & bit) {
                (0, 0) => Color::Empty,
                (_, 0) => Color::Black,
                (0, _) => Color::White,
                (_, _) => return Err(BoardError::InvalidState),
            };
        }
        Ok(board_vec)
    }

    /// Get the current board state as a vector of colors
    /// # Returns
    /// * Vector of colors
    /// # Note
    /// * Color::Black: black's stone
    /// * Color::White: white's stone
    /// * Color::Empty: empty
    pub fn get_board_vec_turn(&self) -> Result<Vec<Color>, BoardError> {
        let mut board_vec = vec![Color::Empty; BOARD_SIZE * BOARD_SIZE];
        let player_color = match self.turn {
            Turn::Black => Color::Black,
            Turn::White => Color::White,
        };
        let opponent_color = player_color.opposite();
        for (i, board_vec_elem) in board_vec.iter_mut().enumerate() {
            let bit = BITS[i];
            *board_vec_elem = match (self.player_board & bit, self.opponent_board & bit) {
                (0, 0) => Color::Empty,
                (_, 0) => player_color,
                (0, _) => opponent_color,
                (_, _) => return Err(BoardError::InvalidState),
            };
        }
        Ok(board_vec)
    }

    /// Get the current board state as a matrix of colors
    /// # Returns
    /// * Matrix shape of (3, 8, 8)
    /// # Note
    /// * fist axis: 0: player's stone, 1: opponent's stone, 2: empty
    /// * second axis: row
    /// * third axis: column
    pub fn get_board_matrix(&self) -> Result<Vec<Vec<Vec<i32>>>, BoardError> {
        let mut board_matrix = vec![vec![vec![0; BOARD_SIZE]; BOARD_SIZE]; 3];
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                let i = x * BOARD_SIZE + y;
                let bit = BITS[i];
                match (self.player_board & bit, self.opponent_board & bit) {
                    (0, 0) => board_matrix[2][x][y] = 1,
                    (_, 0) => board_matrix[0][x][y] = 1,
                    (0, _) => board_matrix[1][x][y] = 1,
                    (_, _) => return Err(BoardError::InvalidState),
                }
            }
        }
        Ok(board_matrix)
    }

    #[inline]
    /// Get the number of player's stones
    pub fn player_piece_num(&self) -> i32 {
        self.player_board.count_ones() as i32
    }

    #[inline]
    /// Get the number of opponent's stones
    pub fn opponent_piece_num(&self) -> i32 {
        self.opponent_board.count_ones() as i32
    }

    /// Get the number of black's stones
    pub fn black_piece_num(&self) -> i32 {
        if self.turn == Turn::Black {
            self.player_piece_num()
        } else {
            self.opponent_piece_num()
        }
    }

    /// Get the number of white's stones
    pub fn white_piece_num(&self) -> i32 {
        if self.turn == Turn::White {
            self.player_piece_num()
        } else {
            self.opponent_piece_num()
        }
    }

    /// Get the sum of all stones
    pub fn piece_sum(&self) -> i32 {
        self.player_piece_num() + self.opponent_piece_num()
    }

    /// Get the difference of player's stones and opponent's
    pub fn diff_piece_num(&self) -> i32 {
        self.player_piece_num() - self.opponent_piece_num()
    }

    #[inline]
    fn get_legal_partial(watch: u64, player_board: u64, shift: usize) -> u64 {
        let mut flip_l = (player_board << shift) & watch;
        let mut flip_r = (player_board >> shift) & watch;
        flip_l |= (flip_l << shift) & watch;
        flip_r |= (flip_r >> shift) & watch;
        let watch_l = watch & (watch << shift);
        let watch_r = watch & (watch >> shift);
        let shift2 = shift + shift;
        flip_l |= (flip_l << shift2) & watch_l;
        flip_r |= (flip_r >> shift2) & watch_r;
        flip_l |= (flip_l << shift2) & watch_l;
        flip_r |= (flip_r >> shift2) & watch_r;
        (flip_l << shift) | (flip_r >> shift)
    }

    fn get_legal_moves_non_avx(&mut self) -> u64 {
        let mask = 0x7E_7E_7E_7E_7E_7E_7E_7E & self.opponent_board;
        let legal_moves = (Board::get_legal_partial(mask, self.player_board, 1)
            | Board::get_legal_partial(self.opponent_board, self.player_board, 8)
            | Board::get_legal_partial(mask, self.player_board, 9)
            | Board::get_legal_partial(mask, self.player_board, 7))
            & !(self.player_board | self.opponent_board);
        self.legal_moves_cache = Some(legal_moves);
        legal_moves
    }

    #[target_feature(enable = "avx2")]
    unsafe fn get_moves_avx(pppp: __m256i, oooo: __m256i) -> u64 {
        let shift1897 = _mm256_set_epi64x(7, 9, 8, 1);

        // Apply edge masks to opponent's board to avoid wrap-around effects
        let moo = _mm256_and_si256(
            oooo,
            _mm256_set_epi64x(
                0x007E7E7E7E7E7E00, // Diagonal mask
                0x007E7E7E7E7E7E00, // Diagonal mask
                0x00FFFFFFFFFFFF00, // Vertical mask
                0x7E7E7E7E7E7E7E7E, // Horizontal mask
            ),
        );

        // Calculate occupied squares
        let occupied = _mm_or_si128(_mm256_castsi256_si128(pppp), _mm256_castsi256_si128(oooo));

        // Find potential flips in left direction
        let mut flip_l = _mm256_and_si256(moo, _mm256_sllv_epi64(pppp, shift1897));
        // Find potential flips in right direction
        let mut flip_r = _mm256_and_si256(moo, _mm256_srlv_epi64(pppp, shift1897));

        // Extend flips one step further
        flip_l = _mm256_or_si256(
            flip_l,
            _mm256_and_si256(moo, _mm256_sllv_epi64(flip_l, shift1897)),
        );
        flip_r = _mm256_or_si256(
            flip_r,
            _mm256_and_si256(moo, _mm256_srlv_epi64(flip_r, shift1897)),
        );

        // Calculate pre-computed masks for further extension
        let pre_l = _mm256_and_si256(moo, _mm256_sllv_epi64(moo, shift1897));
        let pre_r = _mm256_srlv_epi64(pre_l, shift1897);

        // Double shift amount for faster propagation
        let shift2 = _mm256_add_epi64(shift1897, shift1897);

        // Extend flips by double steps
        flip_l = _mm256_or_si256(
            flip_l,
            _mm256_and_si256(pre_l, _mm256_sllv_epi64(flip_l, shift2)),
        );
        flip_r = _mm256_or_si256(
            flip_r,
            _mm256_and_si256(pre_r, _mm256_srlv_epi64(flip_r, shift2)),
        );

        // Extend flips by double steps once more
        flip_l = _mm256_or_si256(
            flip_l,
            _mm256_and_si256(pre_l, _mm256_sllv_epi64(flip_l, shift2)),
        );
        flip_r = _mm256_or_si256(
            flip_r,
            _mm256_and_si256(pre_r, _mm256_srlv_epi64(flip_r, shift2)),
        );

        // Calculate move positions from flips
        let mm = _mm256_or_si256(
            _mm256_sllv_epi64(flip_l, shift1897),
            _mm256_srlv_epi64(flip_r, shift1897),
        );

        // Combine 256-bit result into 128-bit
        let m = _mm_or_si128(_mm256_castsi256_si128(mm), _mm256_extracti128_si256(mm, 1));

        // Select only empty squares and combine the results
        let result = _mm_andnot_si128(occupied, _mm_or_si128(m, _mm_unpackhi_epi64(m, m)));
        _mm_cvtsi128_si64(result) as u64
    }

    #[target_feature(enable = "avx2")]
    unsafe fn get_legal_moves_avx2(&mut self) -> u64 {
        let player_vector = _mm256_set_epi64x(
            self.player_board as i64,
            self.player_board as i64,
            self.player_board as i64,
            self.player_board as i64,
        );
        let opponent_vector = _mm256_set_epi64x(
            self.opponent_board as i64,
            self.opponent_board as i64,
            self.opponent_board as i64,
            self.opponent_board as i64,
        );

        let legal_moves = Board::get_moves_avx(player_vector, opponent_vector);
        self.legal_moves_cache = Some(legal_moves);
        legal_moves
    }

    /// Get the legal moves for the player as a bitboard
    pub fn get_legal_moves(&mut self) -> u64 {
        if let Some(legal_moves) = self.legal_moves_cache {
            return legal_moves;
        }
        if is_x86_feature_detected!("avx2") {
            unsafe { self.get_legal_moves_avx2() }
        } else {
            self.get_legal_moves_non_avx()
        }
    }

    /// Get the legal moves for the player as a vector of positions
    pub fn get_legal_moves_vec(&mut self) -> StackVec64<usize> {
        let legal_moves = self.get_legal_moves();
        let mut legal_moves_vec = StackVec64::new();
        for (i, &bit) in BITS.iter().enumerate() {
            if legal_moves & bit != 0 {
                legal_moves_vec.push(i);
            }
        }
        legal_moves_vec
    }

    /// Get the legal moves for the player as a vector of boolean
    /// * true: legal move, false: illegal move
    pub fn get_legal_moves_tf(&mut self) -> Vec<bool> {
        let legal_moves = self.get_legal_moves();
        let mut legal_moves_tf = Vec::with_capacity(BOARD_SIZE * BOARD_SIZE);
        for &bit in BITS.iter() {
            legal_moves_tf.push(legal_moves & bit != 0);
        }
        legal_moves_tf
    }

    /// Get if the move is legal
    pub fn is_legal_move(&mut self, pos: usize) -> bool {
        self.get_legal_moves() & BITS[pos] != 0
    }

    /// Get the list of board states after legal moves
    pub fn get_child_boards(&mut self) -> Option<Vec<Board>> {
        if self.is_pass() {
            return None;
        }
        let legal_moves = self.get_legal_moves();
        let mut child_boards = Vec::with_capacity(legal_moves.count_ones() as usize);
        for (i, &bit) in BITS.iter().enumerate() {
            if legal_moves & bit != 0 {
                let mut child_board = self.clone();
                child_board.do_move(i).unwrap();
                child_boards.push(child_board);
            }
        }
        Some(child_boards)
    }

    fn reverse_non_avx(&mut self, pos: u64) {
        let mut reversed: u64 = 0;
        // tmp is position of stones to reverse if piece exists on the end of stones to reverse
        // mask is position that exists opponent's stone to reverse from piece on each direction
        macro_rules! get_reverse_l {
            ($mask:expr, $dir:expr) => {
                let mut mask = $mask & (pos << $dir);
                let mut tmp = 0;
                while mask & self.opponent_board != 0 {
                    tmp |= mask;
                    mask = $mask & (mask << $dir);
                }
                if (mask & self.player_board) != 0 {
                    reversed |= tmp;
                }
            };
        }
        macro_rules! get_reverse_r {
            ($mask:expr, $dir:expr) => {
                let mut mask = $mask & (pos >> $dir);
                let mut tmp = 0;
                while mask & self.opponent_board != 0 {
                    tmp |= mask;
                    mask = $mask & (mask >> $dir);
                }
                if (mask & self.player_board) != 0 {
                    reversed |= tmp;
                }
            };
        }
        get_reverse_l!(0xFE_FE_FE_FE_FE_FE_FE_FE, 1); // left
        get_reverse_l!(0xFF_FF_FF_FF_FF_FF_FF_00, 8); // up
        get_reverse_l!(0xFE_FE_FE_FE_FE_FE_FE_00, 9); // upper left
        get_reverse_l!(0x7F_7F_7F_7F_7F_7F_7F_00, 7); // upper right
        get_reverse_r!(0x7F_7F_7F_7F_7F_7F_7F_7F, 1); // right
        get_reverse_r!(0x00_FF_FF_FF_FF_FF_FF_FF, 8); // down
        get_reverse_r!(0x00_7F_7F_7F_7F_7F_7F_7F, 9); // lower right
        get_reverse_r!(0x00_FE_FE_FE_FE_FE_FE_FE, 7); // lower left
        self.player_board ^= reversed | pos;
        self.opponent_board ^= reversed;
    }

    #[target_feature(enable = "avx2")]
    unsafe fn mm_flip(&self, pos: usize) -> u64 {
        // Create a single 128-bit vector with player in low bits, opponent in high bits
        let op = _mm_set_epi64x(self.opponent_board as i64, self.player_board as i64);
        let pppp = _mm256_broadcastq_epi64(op);
        let oooo = _mm256_broadcastq_epi64(_mm_unpackhi_epi64(op, op));

        // Access LRMASK properly as two 256-bit values
        let mask0 = _mm256_loadu_si256(LRMASK[pos].v.as_ptr() as *const __m256i);
        let mask1 = _mm256_loadu_si256(LRMASK[pos].v.as_ptr().add(4) as *const __m256i);

        // Right direction processing
        let rp = _mm256_and_si256(pppp, mask0);
        let mut rs = _mm256_or_si256(rp, _mm256_srlv_epi64(rp, _mm256_set_epi64x(7, 9, 8, 1)));
        rs = _mm256_or_si256(rs, _mm256_srlv_epi64(rs, _mm256_set_epi64x(14, 18, 16, 2)));
        rs = _mm256_or_si256(rs, _mm256_srlv_epi64(rs, _mm256_set_epi64x(28, 36, 32, 4)));

        let re = _mm256_xor_si256(_mm256_andnot_si256(oooo, mask0), rp);
        let mut flip = _mm256_and_si256(_mm256_andnot_si256(rs, mask0), _mm256_cmpgt_epi64(rp, re));

        // Left direction processing
        let mut lo = _mm256_andnot_si256(oooo, mask1);
        lo = _mm256_and_si256(
            _mm256_xor_si256(_mm256_add_epi64(lo, _mm256_set1_epi64x(-1)), lo),
            mask1,
        );

        let lf = _mm256_andnot_si256(pppp, lo);
        flip = _mm256_or_si256(flip, _mm256_andnot_si256(_mm256_cmpeq_epi64(lf, lo), lf));

        // Combine results
        let res128 = _mm_or_si128(
            _mm256_castsi256_si128(flip),
            _mm256_extracti128_si256(flip, 1),
        );
        _mm_extract_epi64(res128, 0) as u64 | _mm_extract_epi64(res128, 1) as u64
    }

    #[target_feature(enable = "avx2")]
    unsafe fn reverse_avx2(&mut self, pos: u64) {
        let position = pos.trailing_zeros() as usize;

        // Get flipped discs using mm_flip
        let flipped_bits = self.mm_flip(position);

        // Update board state
        self.player_board ^= flipped_bits | pos;
        self.opponent_board ^= flipped_bits;
    }

    /// Reverse the stones
    /// # Arguments
    /// * `pos` - Position to place the stone as a bitboard
    pub fn reverse(&mut self, pos: u64) {
        if is_x86_feature_detected!("avx2") {
            unsafe {
                self.reverse_avx2(pos);
            }
        } else {
            self.reverse_non_avx(pos);
        }
    }

    /// Place the stone
    /// # Arguments
    /// * `pos` - Position to place the stone
    /// # Returns
    /// * `Result<(), BoardError>` - Ok(()) if successful, Err(BoardError) otherwise
    /// # Note
    /// * If the move is illegal, return Err(BoardError::InvalidMove)
    /// * If the position is invalid, return Err(BoardError::InvalidPosition)
    pub fn do_move(&mut self, pos: usize) -> Result<(), BoardError> {
        if pos >= BOARD_SIZE * BOARD_SIZE {
            return Err(BoardError::InvalidPosition);
        }
        let pos_bit = BITS[pos];
        if self.is_legal_move(pos) {
            self.reverse(pos_bit);
            swap(&mut self.player_board, &mut self.opponent_board);
            self.turn = self.turn.opposite();
            self.legal_moves_cache = None;
        } else {
            return Err(BoardError::InvalidMove);
        }
        Ok(())
    }

    /// Pass the turn
    /// # Returns
    /// * `Result<(), BoardError>` - Ok(()) if successful, Err(BoardError) otherwise
    /// # Note
    /// * If there is a legal move, return Err(BoardError::InvalidPass)
    /// * If the game is over, return Err(BoardError::InvalidPass)
    pub fn do_pass(&mut self) -> Result<(), BoardError> {
        if !self.is_pass() || self.is_game_over() {
            return Err(BoardError::InvalidPass);
        }
        swap(&mut self.player_board, &mut self.opponent_board);
        self.turn = self.turn.opposite();
        self.legal_moves_cache = None;
        Ok(())
    }

    #[inline]
    /// Get if the player must pass the turn
    /// # Returns
    /// * true: must pass, false: must not pass
    /// # Note
    /// * If there is a legal move, return false
    /// * If the game is over, return false
    pub fn is_pass(&self) -> bool {
        if let Some(legal_moves) = self.legal_moves_cache {
            return legal_moves == 0;
        }
        let mask_v = 0x7E_7E_7E_7E_7E_7E_7E_7E & self.opponent_board;
        let mask_h = 0x00_FF_FF_FF_FF_FF_FF_00 & self.opponent_board;
        let mask_a = 0x00_7E_7E_7E_7E_7E_7E_00 & self.opponent_board;
        let enmpy = !(self.player_board | self.opponent_board);
        if Board::get_legal_partial(mask_v, self.player_board, 1) & enmpy != 0 {
            return false;
        }
        if Board::get_legal_partial(mask_h, self.player_board, 8) & enmpy != 0 {
            return false;
        }
        if Board::get_legal_partial(mask_a, self.player_board, 9) & enmpy != 0 {
            return false;
        }
        if Board::get_legal_partial(mask_a, self.player_board, 7) & enmpy != 0 {
            return false;
        }
        true
    }

    /// Get if the game is over
    /// # Returns
    /// * true: game over, false: game not over
    pub fn is_game_over(&self) -> bool {
        if self.is_pass() {
            let opponent_board = Board {
                player_board: self.opponent_board,
                opponent_board: self.player_board,
                turn: self.turn.opposite(),
                legal_moves_cache: None,
            };
            if opponent_board.is_pass() {
                return true;
            }
        }
        false
    }

    /// Get if the player wins
    /// # Note
    /// * If the game is not over, return Err(BoardError::GameNotOverYet)
    pub fn is_win(&self) -> Result<bool, BoardError> {
        if self.is_game_over() {
            Ok(self.player_piece_num() > self.opponent_piece_num())
        } else {
            Err(BoardError::GameNotOverYet)
        }
    }

    /// Get if the player loses
    /// # Note
    /// * If the game is not over, return Err(BoardError::GameNotOverYet)
    pub fn is_lose(&self) -> Result<bool, BoardError> {
        if self.is_game_over() {
            Ok(self.player_piece_num() < self.opponent_piece_num())
        } else {
            Err(BoardError::GameNotOverYet)
        }
    }

    /// Get if the game is draw
    /// # Note
    /// * If the game is not over, return Err(BoardError::GameNotOverYet)
    pub fn is_draw(&self) -> Result<bool, BoardError> {
        if self.is_game_over() {
            Ok(self.player_piece_num() == self.opponent_piece_num())
        } else {
            Err(BoardError::GameNotOverYet)
        }
    }

    /// Get if the black wins
    /// # Note
    /// * If the game is not over, return Err(BoardError::GameNotOverYet)
    pub fn is_black_win(&self) -> Result<bool, BoardError> {
        if self.is_game_over() {
            Ok(self.black_piece_num() > self.white_piece_num())
        } else {
            Err(BoardError::GameNotOverYet)
        }
    }

    /// Get if the white wins
    /// # Note
    /// * If the game is not over, return Err(BoardError::GameNotOverYet)
    pub fn is_white_win(&self) -> Result<bool, BoardError> {
        if self.is_game_over() {
            Ok(self.white_piece_num() > self.black_piece_num())
        } else {
            Err(BoardError::GameNotOverYet)
        }
    }

    /// Get the winner
    /// # Returns
    /// * `Result<Option<Turn>, BoardError>`
    /// # Note
    /// * If the game is not over, return Err(BoardError::GameNotOverYet)
    /// * If the game is draw, return None
    /// * Otherwise, return the winner
    pub fn get_winner(&self) -> Result<Option<Turn>, BoardError> {
        if self.is_game_over() {
            if self.is_win().unwrap() {
                Ok(Some(self.turn))
            } else if self.is_lose().unwrap() {
                Ok(Some(self.turn.opposite()))
            } else {
                Ok(None)
            }
        } else {
            Err(BoardError::GameNotOverYet)
        }
    }

    /// Get random move
    /// # Returns
    /// * `Result<usize, BoardError>`
    /// # Note
    /// * If there is no legal move, return Err(BoardError::NoLegalMove)
    pub fn get_random_move(&mut self) -> Result<usize, BoardError> {
        let legal_moves_vec = self.get_legal_moves_vec();
        if legal_moves_vec.is_empty() {
            return Err(BoardError::NoLegalMove);
        }
        let random_index = rand::random::<usize>() % legal_moves_vec.len();
        Ok(legal_moves_vec[random_index])
    }

    /// Convert the board state to a string
    /// # Returns
    /// * String representation of the board
    /// # Note
    /// * X: Black, O: White
    /// * this is used for fmt::Display
    pub fn to_string(&self) -> Result<String, BoardError> {
        let mut board_str = String::new();
        let player_char = match self.turn {
            Turn::Black => LINE_CHAR_BLACK,
            Turn::White => LINE_CHAR_WHITE,
        };
        let opponent_char = match self.turn {
            Turn::Black => LINE_CHAR_WHITE,
            Turn::White => LINE_CHAR_BLACK,
        };
        board_str.push_str(" |abcdefgh\n-+--------\n");
        for i in 0..BOARD_SIZE {
            board_str.push_str(&format!("{}|", i + 1));
            for j in 0..BOARD_SIZE {
                let pos = BITS[i * BOARD_SIZE + j];
                match (self.player_board & pos, self.opponent_board & pos) {
                    (0, 0) => board_str.push(LINE_CHAR_EMPTY),
                    (_, 0) => board_str.push(player_char),
                    (0, _) => board_str.push(opponent_char),
                    (_, _) => return Err(BoardError::InvalidState),
                }
            }
            board_str.push('\n');
        }
        Ok(board_str)
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string().unwrap())
    }
}
