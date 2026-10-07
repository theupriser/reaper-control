#!/bin/zsh
# Runs every method x lead x repeat against the isolated test REAPER and waits for each run to end.
RC2="${1:?usage: run_matrix.sh <resource>/RC2 [repeats]}"; REPEATS="${2:-3}"
for method in seek stopseek; do
  for lead in 0 15 30 45; do
    for n in $(seq $REPEATS); do
      echo "$method $lead" > "$RC2/s2-run.txt"
      while [ -e "$RC2/s2-run.txt" ] || ! tail -n1 "$RC2/spike-s2.log" | grep -q '^END'; do sleep 0.5; done
      echo "done: $method lead=$lead #$n"
    done
  done
done
