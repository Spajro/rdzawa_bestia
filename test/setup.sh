if [ ! -e chess_engine_evaluator ]; then
  git clone https://github.com/Spajro/chess_engine_evaluator
fi

if [ "$(uname -s)" == "Linux"  ] && [ ! -e stockfish ]; then
  wget https://github.com/official-stockfish/Stockfish/releases/latest/download/stockfish-ubuntu-x86-64-avx2.tar
  tar -xf stockfish-ubuntu-x86-64-avx2.tar
  rm stockfish-ubuntu-x86-64-avx2.tar
fi

if [ "$(uname -s)" == "Linux"  ]; then
  python3 -m venv .venv
  source .venv/bin/activate
  which python
fi

python3 -m pip install zstandard
python3 -m pip install stockfish
python3 -m pip install chess

python3 chess_engine_evaluator/setup.py stockfish stockfish/stockfish-ubuntu-x86-64-avx2
python3 chess_engine_evaluator/setup.py uci rdzawa_bestia ../target/debug/rdzawa_bestia
python3 chess_engine_evaluator/setup.py puzzles

deactivate