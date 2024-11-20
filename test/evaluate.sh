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

if [ "$machine" == Linux ]; then
  source .venv/bin/activate
  which python
fi

if [ "$machine" == MinGw  ]; then
    source .venv\\Scripts\\activate
    where python
    alias python3=py
fi

python3 chess_engine_evaluator/evaluate.py rdzawa_bestia --threads=10 --length=medium --verbose=result

deactivate
shopt -u expand_aliases
