mod book;
mod eval;
mod search;
mod uci;

use uci::UciHandler;

fn main() {
    let mut uci = UciHandler::new();
    uci.run();
}
