To use the software, you must first ensure there are some prerequisite software installed on your computer. First, you need to choose your terminal application. If you are on macOS, you already have the Terminal application installed. If you're on Unix system, you've got your own default flavor as well. If you're on Windows, you might be able to provide more relevant information than me but [this article looks relatively up to date.](https://towardsdatascience.com/new-windows-terminal-the-best-you-can-have-9945294707e7). The terminal interface is what the software is built around.

Next, download the software by opening your terminal, typing the following and pressing enter:

```
git clone git@github.com:jago-community/jago.git ~/local/jago
```

Next, you need to choose your text editor. There are plenty to choose from but the software provides an integration with the Vim text editor. To install Vim on macOS, type the following into your terminal and press enter:

```
brew install vim
```

To install Vim on a Unix system other than macOS, find instructions for your specific flavor and if you'd like, add them to this file.

To install Vim on Windows, [this article looks up to date.](https://www.freecodecamp.org/news/vim-windows-install-powershell/) If you can come up with easier instructions, please contribute them here.

The software is is written in the rust programming language. Because of this, you need to install the Rust programming language. Please follow the instructions on the [rust language's installation page.](https://www.rust-lang.org/learn/get-started) Their documentation is usually approachable but if you have any questions or problems, please reach out to [contributors@jago.cafe.](mailto:contributors@jago.cafe)

Once you have rust and its suite of tools installed, change the context of your terminal to the software's directory:

```
cd ~/local/jago
```

Next, you need to build and install your own version of the software. Because you are building and installing your own version and not downloading an already built version and installing it, this will take a fair amount of time longer than you might expect. I suggest getting to know your editor by opening another terminal window and typing the following and pressing enter:

```
vimtutor
```

If you already know the editor well, poke around the software while you wait. Another idea is to pick random items out of the dependency list you see are being installed in the process and looking them up on [rust's software sharing site.](https://crates.io)

To build and install the software, type the following into your terminal and press enter:

```
cargo install --path .
```

Once you have the software installed and you feel like you know how to use your editor by typing the following and pressing enter:

```
vim
```

Next, we'll open your configuration file by pressing the following keys and followed by enter:

```
:e $MYVIMRC
```

Next, you need to enter **insert** mode and add the following line to the file by pressing `i` and then typing:

```
source ~/local/jago/instrument.vim
```

Exit insert mode by pressing the escape key and save **and** close the editor by typing `:w`. From now on, when you open the editor it will be instrumented with tools to make the editor more powerful provided by the software. However, we need to reload this current session for these tools to be available to you now. To do this, type the following and press enter:

```
:source %
```

Vim provides a shortcut in this prompt for the current file which is `%` so this line reads your configuration file and makes changes to your editor accordingly.

Now, let's open the exact file you are reading right now. To do this, from **normal** mode type the following and press enter:

```
:e ~/local/jago/start
```
