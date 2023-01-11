use tetron::*;

use rand::Rng;
use std::{thread, time::Duration};
use std::io::{self, Write};

use super::*;

fn spawn_cheese (field: &mut Field, board: &mut Board) {
    static mut prev: u8 = 0;

    let mut rng = rand::thread_rng();
    let i: u8 = unsafe {
        let mut i: u8 = prev;
        while i == prev { i = rng.gen_range(0..10); }
        prev = i;
        i
    };
    let nrow: u16 = ((1 << 10) - 1) - (1 << i);

    for y in 1..20 {
        field.m[y-1] = field.m[y];
        board.m[y-1] = board.m[y];
    }
    field.m[19] = nrow;
    for x in 0..10 {
        board.m[19][x] = if nrow & 1 << x > 0 {Piece::L} else {Piece::None};
    }
}

pub fn cheese_sandbox (lines: usize, log: bool) -> (u32, u64, u128) {
    let mut state = State::new();
    let mut board = Board::new(Some(&state.field));

    // Gen cheese 
    let mut cheese_row = 10.min(lines);
    let mut cheese_clears = 0;
    let mut piece_cnt: u32 = 0;
    for _ in (20 - lines.min(10))..20 {
        spawn_cheese(&mut state.field, &mut board);
    }

    let mut bag = vec![Piece::J, Piece::L, Piece::S, Piece::Z, Piece::T, Piece::I, Piece::O];
    
    let mut avg_dt: u128 = 0;
    let mut total_dt: u64 = 0;
    while state.pieces.len() < 6 {
        state.pieces.push_back(sandbox::draw(&mut bag));
    }
    println!("{}", board);
    while cheese_clears < lines {
        // Draw pieces
        while state.pieces.len() < 6 {
            state.pieces.push_back(sandbox::draw(&mut bag));
        }
        
        // Solve & Bench
        let start = Instant::now();        
        if let Some(out) = solve(&state, 2, Some(EvaluatorMode::DS)) {
            let dt = start.elapsed().as_micros();
            total_dt += (dt / 1_000_100) as u64;
            avg_dt = if avg_dt == 0 {dt} else {(avg_dt + dt) / 2};

            // Apply move to colored board
            board.apply_move(&out.1, &state.pieces[0], if state.hold == Piece::None {&state.pieces[1]} else {&state.hold});

            // Log out result
            if log {
                println!("Time consumed: {}{}{}us", BLD, dt, RST);
                println!("Avg benchmark: {}{}{}us", BLD, avg_dt, RST);
                sandbox::render(&board, &out.0);
            }
            // Process cheese
            for y in (20-cheese_row)..20 {
                if out.0.props.clears & 1 << y > 0 {
                    cheese_row -= 1;
                    cheese_clears += 1;
                }
            }
            state.field = out.0.field;
            state.pieces = out.0.pieces;
            state.hold = out.0.hold;

            // Spawn Chese 
            if out.0.props.clears == 0 && cheese_row < 10 && cheese_clears + cheese_row < lines {
                while cheese_row < 10 {
                    spawn_cheese(&mut state.field, &mut board);
                    cheese_row += 1;
                }
            }
        } else {
            println!("{BLD}No results found, game over.{RST}");
            break;
        }
        print!(".");
        io::stdout().flush().unwrap();
        piece_cnt += 1;
        thread::sleep(Duration::from_millis(100));
    }
    println!();
    (piece_cnt, total_dt, avg_dt)
}

pub fn cheese_exam (iter: usize, lines: usize, log: bool) {
    println!("{HLT}--CHEESE EXAM--{RST}");
    println!("{BLD}--iters: {iter}, lines: {lines}--{RST}");

    let mut pieces_res: (f64, u32, u32) = (0.0, 0, u32::MAX);
    let mut time_res: (f64, u64, u64) = (0.0, 0, 0);
    let mut avg_pps: f64 = 0.0;

    for i in 0..iter {
        let (pieces, total_dt, avg_dt) = cheese_sandbox(lines, log);
        let pps: f64 = 1.0 / (avg_dt as f64 / 1_000_000.0);

        pieces_res.0 = if pieces_res.0 == 0.0 {pieces as f64} else {(pieces_res.0 + pieces as f64) / 2.0};
        pieces_res.1 = pieces_res.1.max(pieces);
        pieces_res.2 = pieces_res.2.min(pieces);

        time_res.0 = if time_res.0 == 0.0 {total_dt as f64} else {(time_res.0 + total_dt as f64) / 2.0};
        time_res.1 = time_res.1.max(total_dt);
        time_res.2 = time_res.2.min(total_dt);

        avg_pps = if avg_pps == 0.0 {pps} else {(avg_pps + pps) / 2.0};

        println!("{BLD}Results{RST}: pieces: {HLT}{}{RST}, time: {BLD}{}{RST}, pps: {BLD}{:.2}{RST}", pieces, total_dt, pps);
    }
    println!("{BLD}Final Results{RST}:");
    println!("avg pieces: {HLT}{}{RST}, worst: {HLT}{}{RST}, best: {BLD}{}{RST}", pieces_res.0, pieces_res.1, pieces_res.2);
    println!("avg time: {HLT}{}{RST}, worst: {HLT}{}{RST}, best: {BLD}{}{RST}", time_res.0, time_res.1, time_res.2);
    println!("avg pps: {HLT}{:.2}{RST}", avg_pps);
}