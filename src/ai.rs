//have arg to run it with AI. maybe button in game? maybe have it compete against player? 

//run in separate thread
//two way communication, rx and tx for both.
//ai sits in a for rx loop. called each time a board is passed                                      PASS RIGHT AFTER UPDATE    MAYBE USE BOARD.GET_BOARD() METHOD
//if piece == spawn location then calc board and set a list of desired moves and coords             WHAT ABOUT AFTER PIECE SET WHEN PIECE SPAWNED BUT BOARD NOT UPDATED? WHEN UPDATED IT DROPS PIECE
//loop and send move_list.next()                        SET DELAY ON SENDING FOR DIFFICULTY         DECOUPLE FROM UPDATE TO ALLOW SPEED         WHAT IF IT DESYNCS A PIECE?

//on each input update check for input from ai thread

//check every rotation at every space               LAZY
//for determining move value
//  iter rows in reverse
//  get height
//  get num of completed lines
//  get holes (if block empty check block above)
//  convert to itering columns then: 
//      get average height of each column then analyse variation 
//      avoid gaps more than 4 tall                                                 ENCOURAGE GAPS 4 TALL?? FOR TETS
//weight of each parameter val is a -1-1 float.                                     USE GENERATIONAL ALG TO TEST ON VERSION WITH NO DELAY
//HOW TO GET MOVE VALUE?
//also check held piece. if no held piece then check next piece
//choose best move out of them all

//goal: have a function that takes a board and returns a series of moves
//  parse and convert board
//  scan for moves. return coords, rotation, and if hold piece
//  find best location
//  generate moves to get to coords, rotation, target piece

//  input handler that gives move_list.next() when asked                DELAY HANDLED BY AI? WORRY ABOUT IT AFTER TRAINING. FOR NOW GO FOR ONE INPUT PER FRAME


//CHANGE TETRIS CODE AS LITTLE AS POSSIBLE

//GEN ALG
//  play game til gameover          ONE INPUT PER FRAME
//  use score to calc               MAYBE AIM FOR LOWER LEVELS TOO? TO ENCOURAGE TETS INSTEAD OF ONE LINE MATCHES

//CANT DO FANCY LAST SECOND MOVES