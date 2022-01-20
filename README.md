[![License](https://img.shields.io/badge/license-MIT-green)](./LICENSE)
[![Heroku](https://heroku-badge.herokuapp.com/?app=heroku-badge&style=flat)](https://heroku-badge.herokuapp.com/projects.html)
[![BINUSMAYA Assistant](https://img.shields.io/badge/BINUSMAYA%20Assistant-Invite%20me!-blue)](https://discord.com/api/oauth2/authorize?client_id=921712744749756427&permissions=139855391824&scope=bot)
# BINUSMAYA_Discord_Bot
Discord bot for assisting daily activities in BINUSMAYA as a student from BINUS University

:warning:**This is not an official bot and is made to assist students**

## Features
- Get schedule
- Get session articles/links
- Get progress status of a session
- Get list of classes from your major
- Update student progress for today's session at 00:00 GMT or 07:00 in WIB (VC, Forum, and assignments will not be updated)

## Third party Apps Used
- [Chrome driver](https://chromedriver.chromium.org/downloads)
- [browsermob proxy v.2.1.4](http://bmp.lightbody.net) with [Java 11](https://www.oracle.com/java/technologies/downloads/#java11)
- [Dropbox API](https://www.dropbox.com) for file storage

:heavy_exclamation_mark:**Make sure the apps mentioned above is in the same folder with the application**

## Discord Bot Info
This is more of a bot for individuals rather than guild servers
- prefix: `=`  
- `=help` to get command list  
- `=help [command]` to get command info
- All commands except `=add` can be run in DM and guild

**Note:** if you type a command and there is no message from the bot, then either you typed the wrong command or argument for the command.

## How It Works
When you add the discord bot and want to run the Binus commands, you first need to register using the `=register` command, this is needed to fulfill the request header requirements and also needed to update your student progress. Don't worry, the bot **will not store your email and password**. It uses file to store the data.

## How To Launch
You can fork this project and write 
```sh
$ export DROPBOX_TOKEN=[dropbox token]
$ export GOOGLE_CHROME_SHIM=[chrome binary path]
$ export APPLICATION_ID=[your_bot_id]
$ cargo run
```

## License
[GPL-3.0 License](LICENSE)

## How To Contribute
You can contribute by simply using the app and report or open an issue is you encountered any problems while using it, when opening an issue, please explain the problem you're having clearly.
