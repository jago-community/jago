To set up your computer as a software workspace, we need to ensure there are some prerequisite software installed on your computer.

First, you need to choose your terminal application. If you are on macOS, you already have the Terminal application installed. If you're on Unix system, you've got your own default flavor as well. If you're on Windows, you might be able to provide more relevant information than me but [this article looks relatively up to date.](https://towardsdatascience.com/new-windows-terminal-the-best-you-can-have-9945294707e7). The terminal interface is the best way to work with software.

Next, download this repository by opening your terminal, typing the following and pressing enter:

```
git clone git@github.com:jago-community/jago.git ~/local/jago
```

The default program we use to work on anything is Vim. To install Vim on macOS, type the following into your terminal and press enter:

```
brew install vim
```

To install Vim on Windows, [this article looks up to date.](https://www.freecodecamp.org/news/vim-windows-install-powershell/) If you can come up with easier instructions, please contribute them here.

To install Vim on a Unix system other than macOS, find instructions for your specific flavor and if you'd like, add them to this file.

 rust programming language. Because of this, you need to install the Rust programming language. Please follow the instructions on the [rust language's installation page.](https://www.rust-lang.org/learn/get-started) Their documentation is usually approachable but if you have any questions or problems, please reach out to [contributors@jago.cafe.](mailto:contributors@jago.cafe)

Once you have rust and its suite of tools installed, change the context of your terminal to this respository's directory:

```
cd ~/local/jago
```

If you want to learn about Vim and the basics of how to use it, type the following into your terminal and press enter:

```
vimtutor
```

After you feel like you can do a thing or two with Vim, we need to refer to some instruments that this repositories provides for your workspace. Do this by typing the following in your terminal and pressing enter:

```
vim ~/local/jago/start
```

Now that you're inside the editor, let's configure it.

Open your configuration file by pressing `:` then typing the following and pressing enter:

```
e $MYVIMRC
```

Now, you need to add a new line with a reference to the instrumentation files in this project. To do this, press `o` and then type the following:

```
source $HOME/local/jago/instrument.vim
```

To save the file, press escape and then type `:w` and press enter. To close Vim, type `:q` and press enter. To save and close Vim at the same time, type `:wq` and press enter.
