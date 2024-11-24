namePuzzle="solve_puzzle"
nameEval="play_games"
nameFunction="evaluation_function"

usage(){
  echo "USAGE:"
  echo "./test.sh ${namePuzzle}"
  echo "./test.sh ${nameEval}"
  echo "./test.sh ${nameFunction}"
}

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

if [ $# == 0 ]; then
  usage
  exit 1
fi

if [ "$machine" == Linux ]; then
  source .venv/bin/activate
  which python
elif [ "$machine" == MinGw  ]; then
    source .venv\\Scripts\\activate
    where python
    alias python3=py
else
  echo "Unsupported Machine:${unameOut}"
  exit 1
fi

if [ "$1" == "$nameFunction" ]; then
  python3 chess_engine_evaluator/test_eval.py rdzawa_bestia 1000 --threads=10
elif [ "$1" == "$namePuzzle" ]; then
    python3 chess_engine_evaluator/solve.py rdzawa_bestia 1000 --threads=10 --tags --length
elif [ "$1" == "$nameEval" ]; then
  python3 chess_engine_evaluator/evaluate.py rdzawa_bestia --threads=10 --length=medium --verbose=result
else
  usage
fi

deactivate
shopt -u expand_aliases
sleep 10