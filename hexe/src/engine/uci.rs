use super::*;

use std::io::{self, BufRead};
use std::mem;
use std::str;

use core::color::Color;
use core::mv::Move;
use engine::Limits;
use engine::thread::job::{self, Job};

const WHITE: usize = Color::White as usize;
const BLACK: usize = Color::Black as usize;

macro_rules! name { () => { "Hexe" } }

macro_rules! authors { () => { "Nikolai Vazquez" } }

macro_rules! id {
    ($mac:ident) => {
        concat!("id ", stringify!($mac), " ", $mac!())
    }
}

macro_rules! unknown_command {
    ($cmd:expr) => { println!("Unknown command: {}", $cmd) }
}

impl Default for Limits {
    fn default() -> Limits {
        // Safe because `bool` uses 0 to represent `false`
        unsafe { mem::zeroed() }
    }
}

type UciIter<'a> = str::SplitWhitespace<'a>;

/// Runs the engine via the [Universal Chess Interface][uci] (UCI) protocol.
///
/// [uci]: http://wbec-ridderkerk.nl/html/UCIProtocol.html
pub struct Uci<'a>(&'a mut Engine);

impl<'a> From<&'a mut Engine> for Uci<'a> {
    #[inline]
    fn from(engine: &'a mut Engine) -> Uci<'a> { Uci(engine) }
}

impl<'a> Uci<'a> {
    /// Returns a reference to the underlying engine over which `self` iterates.
    #[inline]
    pub fn engine(&self) -> &Engine { &self.0 }

    /// Returns a mutable reference to the underlying engine over which `self`
    /// iterates.
    #[inline]
    pub fn engine_mut(&mut self) -> &mut Engine { &mut self.0 }

    /// Runs the UCI loop, feeding commands from `stdin`.
    ///
    /// This method retains a lock on `stdin` until it exits. To feed commands
    /// differently, use [`start_with`](#method.start_with).
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,norun
    /// use hexe::engine::Engine;
    ///
    /// let mut engine = Engine::default();
    /// engine.uci().start();
    /// ```
    pub fn start(&mut self) {
        let stdin = io::stdin();
        let lines = stdin.lock().lines().filter_map(Result::ok);
        for line in lines {
            if !self.run_line(&line) {
                break;
            }
        }
    }

    /// Runs the UCI loop, feeding commands from an iterator.
    ///
    /// # Examples
    ///
    /// The UCI can be fed command line arguments.
    ///
    /// ```rust,norun
    /// use hexe::engine::Engine;
    /// use std::env;
    ///
    /// let mut args = env::args();
    /// args.next(); // discard program name
    ///
    /// let mut engine = Engine::default();
    /// engine.uci().start_with(args);
    /// ```
    pub fn start_with<I>(&mut self, commands: I)
        where I: IntoIterator,
              I::Item: AsRef<str>,
    {
        for line in commands {
            self.run(line.as_ref());
        }
    }

    /// Runs a single UCI command or multiple if newlines are found.
    #[inline]
    pub fn run(&mut self, command: &str) {
        if command.is_empty() {
            unknown_command!(command);
        } else {
            for line in command.lines() {
                if !self.run_line(line) {
                    break;
                }
            }
        }
    }

    fn run_line(&mut self, line: &str) -> bool {
        let mut split = line.split_whitespace();
        match split.next().unwrap_or("") {
            "quit"       => return false,
            "uci"        => self.cmd_uci(),
            "stop"       => self.cmd_stop(),
            "ponderhit"  => self.cmd_ponder_hit(),
            "position"   => self.cmd_position(split),
            "setoption"  => self.cmd_set_option(split),
            "ucinewgame" => self.cmd_new_game(),
            "go"         => self.cmd_go(split),
            "isready"    => println!("readyok"),
            _            => unknown_command!(line),
        }
        true
    }

    fn report_options(&self) {
        println!(
            "\noption name Threads type spin default {0} min 1 max {1}\
             \noption name Hash type spin default 1 min 1 max {1}",
            ::num_cpus::get(),
            usize::MAX,
        );
    }

    fn cmd_uci(&self) {
        println!(id!(name));
        println!(id!(authors));
        self.report_options();
        println!("uciok");
    }

    fn cmd_stop(&mut self) {
        self.engine_mut().stop();
    }

    fn cmd_ponder_hit(&mut self) {
        unimplemented!();
    }

    fn cmd_position(&mut self, _: UciIter) {
        unimplemented!();
    }

    fn cmd_set_option(&mut self, mut iter: UciIter) {
        iter.next(); // consume "name"

        let mut name  = String::new();
        let mut value = String::new();

        while let Some(next) = iter.next() {
            if next == "value" {
                break;
            }
            if !name.is_empty() {
                name.push(' ');
            }
            name.push_str(next);
        }

        for next in iter {
            if !value.is_empty() {
                value.push(' ');
            }
            value.push_str(next);
        }

        // Performs a case-insensitive check against the option
        let match_option = |opt: &str| {
            ::util::matches_lower_alpha(opt.as_ref(), name.as_ref())
        };

        if match_option("threads") {
            panic!("Cannot currently set number of threads");
        } else if match_option("hash") {
            match value.parse::<usize>() {
                Ok(value) => {
                    self.0.table.resize_exact(value);
                },
                Err(e) => {
                    // TODO: handle could not parse value
                },
            }
        } else {
            println!("No such option: {}", name);
        }
    }

    fn cmd_new_game(&mut self) {
        unimplemented!();
    }

    fn cmd_go(&mut self, mut iter: UciIter) {
        let mut limits = Limits::default();
        let mut moves  = Vec::<Move>::new();

        macro_rules! update {
            ($val:expr) => {
                if let Some(Ok(val)) = iter.next().map(str::parse) {
                    $val = val
                }
            }
        }

        while let Some(next) = iter.next() {
            match next {
                "searchmoves" => while let Some(m) = iter.next() {
                    if let Some(mv) = self.cmd_read_move(m) {
                        moves.push(mv);
                    }
                },
                "ponder"    => limits.ponder = true,
                "infinite"  => limits.infinite = true,
                "wtime"     => update!(limits.time[WHITE]),
                "btime"     => update!(limits.time[BLACK]),
                "winc"      => update!(limits.inc[WHITE]),
                "binc"      => update!(limits.inc[BLACK]),
                "movestogo" => update!(limits.moves_to_go),
                "depth"     => update!(limits.depth),
                "nodes"     => update!(limits.nodes),
                "mate"      => update!(limits.mate),
                "movetime"  => update!(limits.move_time),
                _ => continue,
            }
        }

        self.cmd_start_thinking(limits, moves.into());
    }

    fn cmd_read_move(&self, s: &str) -> Option<Move> {
        unimplemented!();
    }

    fn cmd_start_thinking(&mut self, limits: Limits, moves: Box<[Move]>) {
        let job = Job::Search { limits, moves };
        self.engine().pool.enqueue(job);
    }
}
