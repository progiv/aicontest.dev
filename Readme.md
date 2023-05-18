# This repo contains aicontest.dev agent
*The code here is not original. First version is just an example-client from official repo https://github.com/bminaiev/aicontest.dev/tree/master/example-client with minor adjustments*
```console
RUST_LOG=info AGENT_PASSWORD=my_secret_password agent --addr=188.166.195.142:7877
```
## Iterations
1. An example client from official repo. simply sets the closest object as the target. Circles around objects.
2. Simple brute force. Iterate through FIRST_STEP_DIRECTIONS, position the target on intersection of direction and a circle around me. Select the target witch will give us the maximum score after MAX_DEPTH iterations. The strategy gives me 5 position on online leaderboard and 10 on highscores.
