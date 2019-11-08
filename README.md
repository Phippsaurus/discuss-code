# Discuss code in `vim` with others

This vim plugin offers support for code annotations and discussions. It is in
its early stages and only offers very basic features which are not formally
tested.

## Setup

`git clone` into `~/.vim/pack/*/start/` and run `./install.sh` in the root directory.

## Commands

At this point the keybindings and colors / symbols are hardcoded - Sorry.

* Mark a code segment in visual mode and press `ac` to enter a comment. Press
  enter to submit.

* In a commented line in normal mode press `sc` to show the respective comment
  in a floating window.

* In a commented line in normal mode press `dc` to delete the respective
  comment.
