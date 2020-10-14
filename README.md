# tetris

A Tetris clone with custom AI.\
Built in rust using my game engine [Untitled_Engine](https://github.com/0rphon/Untitled_Game) and my error handling crate [dynerr](https://github.com/0rphon/dynerr). You can download the compiled windows binary [here](https://drive.google.com/file/d/12WrdRk6TMtHe93KBFNBXWSXnIR8WRBE0/view?usp=sharing).\
\
Everyone knows Tetris...but what about AI powered auto-tetris?\
<img src="player.gif" width="45%" title="Player"/> <img src="ai.gif" width="45%" title="GAI"/>
\
I didnt really set up training for public use, but if you're interested in training your own AI then you can pass the arg --train. By default the console is suppressed on release builds, so if youre training then MAKE SURE to compile in debug mode so you can see the training output. As far as I can tell theres no way to change this behavior in rust. Look at the constants in train.rs to change how the evolutionary alg works. To use an AI you trained yourself use the flag --use_best.\


```
TetrisGAI: Why go through the work of playing tetris when you could just automate it?

--auto-loop:        Allow the AI to restart games on its own.

--train:            Train your own AI. See the constants in train.rs to change how the AI is trained. 
                    Can only be used in debug builds and cant be used with other commands.
                    Still needs work.

--use_best:         Use the top result from training. Stored in the top line of best.log.

--help:             Show this command and exit.
```
