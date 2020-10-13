# tetris

tetris clone with custom AI.\
built in rust using my game engine [Untitled_Engine](https://github.com/0rphon/Untitled_Game) and my error handling crate [Dynerr](https://github.com/0rphon/dynerr). You can download the compiled windows binary [here](https://drive.google.com/file/d/12WrdRk6TMtHe93KBFNBXWSXnIR8WRBE0/view?usp=sharing)\
\
Everyone knows tetris\
![gif of manual gameplay](player.gif)\
\
But what about super powered auto-tetris?\
![gif of AI gameplay](ai.gif)\
\
--auto-loop to let the ai restart games on its own.\
\
i didnt really set it up for public use, but if youre interested in training your own AI then you can pass the arg --train. by default the console is suppressed on release builds, so if youre training then MAKE SURE to either compile in debug mode or comment out the cfg_attr line in main.rs or else you wont be able to see any output! as far as i can tell theres no way to change this behavior in rust.\
i have plans to set up config files for the AI so it can be trained and changed without recompiling, but for now look at the constants in train.rs to change how the evolutionary alg works.\
