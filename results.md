# FINAL RESULT

## Depth 8
best move: Some(ChessMove { source: Square(11), dest: Square(27), promotion: None })
D Second
D Fourth
info Score 0
Evaluation_cnt=49762881
Evaluations per second = 7451623
test minmax_engine::mod_minmax_tests::minmax_depth8_inital_position ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 6.68s

## qdepth=0 depth=6 without AB pruning
running 1 test
best move: Some(ChessMove { source: Square(12), dest: Square(20), promotion: None })
E Second
E Third
info Score -45
Evaluation_cnt=119060679
Evaluations per second = 7151936.5
test minmax_engine::mod_minmax_tests::minmax_depth8_inital_position ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 16.65s

# OLD CODE

## Depth 8
best move: Some(Normal { role: Pawn, from: D2, capture: None, to: D4, promotion: None })
info Score 0
Evaluation_cnt=28604101
Evaluations per second = 4051678.5
test minmax_engine::mod_minmax_tests::minmax_depth8_inital_position ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.06s

## qdepth=0 depth=6 without AB pruning
running 1 test
best move: Some(Normal { role: Pawn, from: E2, capture: None, to: E3, promotion: None })
info Score -45
Evaluation_cnt=119060679
Evaluations per second = 4600433
test minmax_engine::mod_minmax_tests::minmax_depth8_inital_position ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 25.88s