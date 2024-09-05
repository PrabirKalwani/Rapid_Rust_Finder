#!/bin/bash

# This script will log all cd commands by monitoring the shell log

# Define the path to the log file where cd commands will be recorded
LOG_FILE="$HOME/cd_log.txt"

# Define the command to ensure the log file exists
TOUCH_CMD="/usr/bin/touch"

# Ensure the log file exists (create it if it doesn't already)
$TOUCH_CMD "$LOG_FILE"

# Start logging the contents of the log file in the background
# 'tail -f' will keep monitoring the log file and output any new entries added to it
# The '&' at the end makes this process run in the background
tail -f "$LOG_FILE" &


# Below are the steps to configure the custom cd function in the zshrc file

# Override the cd command in zshrc to log usage (this section is commented out)
# function cd() {
#     echo "$(date '+%Y-%m-%d %H:%M:%S') cd $PWD -> $1" >> ~/cd_log.txt
#     builtin cd "$1"
# }

# Define the path to your project and script
# This would be used to call your custom script if necessary
# /Users/prabirkalwani/Data/Programming/os_project/cd_script.sh  &
