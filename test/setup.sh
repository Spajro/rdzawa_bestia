shopt -s expand_aliases

unameOut="$(uname -s)"
case "${unameOut}" in
    Linux*)     machine=Linux;;
    Darwin*)    machine=Mac;;
    CYGWIN*)    machine=Cygwin;;
    MINGW*)     machine=MinGw;;
    MSYS_NT*)   machine=MSys;;
    *)          machine="${unameOut}"
esac

if [ ! "$machine" == Linux ] && [ ! "$machine" == MinGw ]; then
  echo "Unsupported Machine:${unameOut}"
  exit 1
fi

if [ ! -e chess_engine_evaluator ]; then
  git clone https://github.com/Spajro/chess_engine_evaluator
fi

if [ "$machine" == Linux  ] && [ ! -e stockfish ]; then
  wget https://github.com/official-stockfish/Stockfish/releases/latest/download/stockfish-ubuntu-x86-64-avx2.tar
  tar -xf stockfish-ubuntu-x86-64-avx2.tar
  rm stockfish-ubuntu-x86-64-avx2.tar
fi

if [ "$machine" == MinGw  ] && [ ! -e stockfish ]; then
  echo "Stockfish"
  wget https://github.com/official-stockfish/Stockfish/releases/latest/download/stockfish-windows-x86-64-avx2.zip
  unzip stockfish-windows-x86-64-avx2.zip
  rm stockfish-windows-x86-64-avx2.zip
fi

if [ "$machine" == Linux ]; then
  if [ ! -e .venv ]; then
    python3 -m venv .venv
  fi
  source .venv/bin/activate
  which python
  engine_path=../target/debug/rdzawa_bestia
  stockfish_path=stockfish/stockfish-ubuntu-x86-64-avx2
fi

if [ "$machine" == MinGw  ]; then
  if [ ! -e .venv ]; then
      py -m venv .venv
    fi
    source .venv\\Scripts\\activate
    where python
    alias python3=py
    engine_path=../target/debug/rdzawa_bestia.exe
    stockfish_path=stockfish/stockfish-windows-x86-64-avx2
fi

python3 -m pip install zstandard
python3 -m pip install stockfish
python3 -m pip install chess

python3 chess_engine_evaluator/setup.py stockfish $stockfish_path
python3 chess_engine_evaluator/setup.py uci rdzawa_bestia $engine_path
python3 chess_engine_evaluator/setup.py puzzles

deactivate
shopt -u expand_aliases
