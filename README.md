[![License: GPL v3](https://img.shields.io/github/license/BryanEgbert/Binusmaya_Discord_Bot.svg)](https://www.gnu.org/licenses/gpl-3.0)
# BINUSMAYA_Discord_Bot
Discord bot for assisting daily activities in BINUSMAYA as a student from BINUS University

:warning:**This is not an official bot and is made for assisting students in BINUS University**

## Features
- Get schedule
- Get session resources
- Get list of classes from your major
- Send daily notification on the schedule at 00:00
- Update student progress for today's session (VC, Forum, and assignments will not be updated)

## Required Third party Apps
- [Chrome driver](https://chromedriver.chromium.org/downloads)
- [browsermob proxy v.2.1.4](http://bmp.lightbody.net) with [Java 11](https://www.oracle.com/java/technologies/downloads/#java11)

:heavy_exclamation_mark:**Make sure the apps mentioned above is in the same folder with the application**

## Discord Bot Info
prefix: `=`  
`=help` to get command list  
`=help [command]` to get command info

## How To Launch
Executable file:
```sh
$ cd [path-to-app]
$ ./BINUSMAYA_Discord_Bot [chrome_binary_path]
```
For developer:  
You can fork this project and write 
```sh
$ cargo run [chrome_binary_path]
```

:heavy_exclamation_mark:**IMPORTANT:** Make sure you use forward slash instead of backward slash to set the chrome binary path, even on windows

## License
[GPL-3.0 License](LICENSE)

## How To Contribute
You can contribute by simply using the app and report or open an issue is you encountered any problems while using it, when opening an issue, please explain the problem you're having clearly.
