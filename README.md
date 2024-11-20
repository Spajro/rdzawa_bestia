# Rdzawa bestia

Stockfish-inspired chess engine implemented in Rust.

## Features implemented:
### Search:
* Iterative Deepening
* Transposition Table
* Move Ordering:
    * Killer Moves
* Selectivity:
    * Alpha-beta pruning
    * Quiescence search

### Evaluation:


## Dependencies

The project use the [chess](https://crates.io/crates/chess) crate for move generation. 

Additionally there are tools for testing the engine and nnue evaluation:
* [Chess engine evaluator](https://github.com/Spajro/chess_engine_evaluator)
* [NN chess evaluation](https://github.com/Spajro/nn_chess_eval)

For the engine tests you also need to have stockfish installed on your system.

## Installation and setup:

For installation just clone the repository.
After that run the script that sets up the testing environment:

```
./test/setup.sh
```

## Use example

