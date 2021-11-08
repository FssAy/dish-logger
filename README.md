# dish-logger

Lib for the ["dish"](https://github.com/Kameleon-07/dish) project using tokio for faster memory scanning.

**EXPERIMENTAL VERSION**

## Features
- searching for tokens concurrently with [tokio](https://tokio.rs/#tk-lib-runtime)
- support for all discord versions
- simple error handling

## Errors
- **__None [0x00]__** <br> Everything is ok
- **__InvalidArgument [0x01]__** <br> Invalid argument has been passed
- **__DebuggerAttach [0x02]__** <br> Couldn't attach debugger to the process 
- **__OpenProcessFailed [0x03]__** <br> Couldn't open process handle 
- **__NoToken [0x04]__** <br> Couldn't find any token in the memory 
- **__AsyncClosed [0x05] (Async only)__** <br> MPSC channel has been closed due to finished operation 
- **__PageSizeZero [0xF1]__** <br> Critical error (shouldn't happen) 

## Usage
extern function `get_token` will always return a struct `Export`.
This struct is built from 2 attributes. 
First one being a pointer to a string (message) and the second one being a 1 byte dedicated for the error value.

If the error value is `0x00` the message will contain found token. <br>
In other cases message can be a null pointer.

**Versions bellow `1.0.0` are not production ready and should not be used in any public releases!**
