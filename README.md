[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)
[![Heroku](https://heroku-badge.herokuapp.com/?app=heroku-badge&style=flat)](https://heroku-badge.herokuapp.com/projects.html)
# BINUSMAYA_Discord_Bot
Discord bot for assisting daily activities in BINUSMAYA as a student from BINUS University

:warning:**This is not an official bot and is made to assist students**

## Features
- Get schedule
- Get session resources
- Get list of classes from your major
- Send daily notification on the schedule at 00:00 GMT
- Update student progress for today's session (VC, Forum, and assignments will not be updated)

## Third party Apps Used
- [Chrome driver](https://chromedriver.chromium.org/downloads)
- [browsermob proxy v.2.1.4](http://bmp.lightbody.net) with [Java 11](https://www.oracle.com/java/technologies/downloads/#java11)
- [Dropbox API](https://www.dropbox.com) for file storage

:heavy_exclamation_mark:**Make sure the apps mentioned above is in the same folder with the application**

## Discord Bot Info
- prefix: `=`  
- `=help` to get command list  
- `=help [command]` to get command info
- Most commands can be run in DM and guild

## How To Launch
You can fork this project and write 
```sh
$ export DROPBOX_TOKEN=[dropbox token]
$ export GOOGLE_CHROME_SHIM=[chrome binary path]
$ cargo run
```

## License
[GPL-3.0 License](LICENSE)

## How To Contribute
You can contribute by simply using the app and report or open an issue is you encountered any problems while using it, when opening an issue, please explain the problem you're having clearly.
