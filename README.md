# REVERB

Welcome to REVERB: Reworked Ear Virtual Entertainment Remarkable Blayer!

REVERB is a modualr music player for all your music needs.

Please note REVERB is still under development so features may be missing or inclomplete please be patient as we work to develop REVERB, in the mean time feel free to enjoy what is already here!

## Highlights

- solve your multi-platform music playing issues, with the ability to have many platforms (e.g. spotify (coming soon), youtube (coming soon), locally downloaded) in one player in one playlist in one queue.
    - Doesnt have the platform you need? Its totally modular add your own! See Wiki (coming soon) for instructions on how to get started.
- Smarter shuffle (coming soon) allows YOU to choose the parameters and options to controll a probability based shuffle to your standards.
- Many optins for UI:
    - Command line interface
    - GUI(coming soon)
    - Dont like any of the UI options? Its totally modular add your own! See Wiki (coming soon) for instructions on how to get started.


## Overview

REVERB is a project currently under development to better suit your music playing needs. We diddnt like the lack of transparancy and use of other players so we made our own.


### Authors

Amber Orton https://github.com/Amber-Orton
Gwen Cieslik-Kara https://github.com/SixOneFiveZero


## Usage

run the program from the command line:

```bash
./reverb

Starting up... 
Reading config... 
Setting global variables... 
Loading startup data... 
Last played playlist not found, creating default playlist... 
Startup successful!
Please enter command or type 'help' for help.
> play
> pause
> help

    project is a WIP help may be out of date:
    avaliliable commands:
    play (composite): play commands
    pause: used to pause the current song
    help: display this help message
    quit: quit the application
    skip: skip the current song
    playlist (composite): manage playlists
    queue (composite): manage the song queue
    use "<command> help" for more detailed help for composite commands
    source code available at: https://github.com/SixOneFiveZero/reverb
```

## Installation

Download or clone the repository and use cargo to build an executable:

```bash
cargo build
```
Then run the program and REVERB will take you through the short setup.

Requirements: cargo.


## Feedback and Contributing

Feel free to add issues or discussions about any features you would like added or bugs you find we love any new ideas! But bare in mind we are a small team working on this for fun mostly so may or may not get round to it.
If you wish to contribute either fork this repository or send us a message!
