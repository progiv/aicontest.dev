# This repo contains aicontest.dev agent
*The code here is not original. First version is just an example-client from official repo https://github.com/bminaiev/aicontest.dev/tree/master/example-client with minor adjustments*
```console
RUST_LOG=info AGENT_PASSWORD=my_secret_password agent --addr=188.166.195.142:7877
```
## Iterations
1. An example client from official repo. simply sets the closest object as the target. Circles around objects.
2. Simple brute force. Iterate through FIRST_STEP_DIRECTIONS, position the target on intersection of direction and a circle around me. Select the target witch will give us the maximum score after MAX_DEPTH iterations. The strategy gives me 5 position on online leaderboard and 10 on highscores.

## Note on profiling
Use https://www.brendangregg.com/flamegraphs.html

First we need to collect stack samples and see what's inside
```console
perf record --call-graph dwarf _binary_ _args_
perf report
```

Than build and see the flamegraph
```console
git clone git@github.com:brendangregg/FlameGraph.git
perf script | ./FlameGraph/stackcollapse-perf.pl > out.perf-folded
./FlameGraph/flamegraph.pl out.perf-folded > perf.svg
firefox perf.svg
```

Actually I didn't find anything useful. Just made shure that most consuming part is the hot path where I calculate the next realm state. Point::scale method taking a lot(16.36%) in it because of an expensive sqrt function. and the GameState::next_turn consuming 44.26% of cpu cycles. Also we spend a lot cloning GameState, where cloning player names takes significant part, but not needed.
