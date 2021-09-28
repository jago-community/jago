To set up your computer as a software workspace, we need to ensure there are some prerequisite software installed on your computer.

First, you need to choose your terminal application. If you are on macOS, you already have the Terminal application installed. If you're on Unix system, you've got your own default flavor as well. If you're on Windows, you might be able to provide more relevant information than me but [this article looks relatively up to date.](https://towardsdatascience.com/new-windows-terminal-the-best-you-can-have-9945294707e7)

A terminal might seem daunting at first until you realize how it's much easier to set up complex work flows. You're able to use structured language to link together a bunch of tasks rather than having to click around a bunch of magic rectangles. It will definitely take some getting used to. As far as the terminal itself is concerned, it doesn't accept input from the mouse. It is however possible to write programs that run inside a terminal and accept mouse input.

Next, download this project by opening your terminal, typing the following and pressing enter:

```
git clone git@github.com:jago-community/jago.git ~/local/jago
```

The default program we use to work on anything is Vim.

To install Vim on macOS, ensure you have [the homebrew package manager installed](https://brew.sh/) and then type the following into your terminal and press enter:

```
brew install vim
```

To install Vim on Windows, [this article looks up to date.](https://www.freecodecamp.org/news/vim-windows-install-powershell/) If you can come up with easier instructions, please contribute them here.

To install Vim on a Unix system other than macOS, find instructions for your specific flavor and if you'd like, add them to this file.

This project uses the rust programming language as it's main computer interface. Because of this, you need to install the Rust programming language. Please follow the instructions on the [rust language's installation page.](https://www.rust-lang.org/learn/get-started) Their documentation is usually approachable but if you have any questions or problems, please reach out to [isaac@jago.community.](mailto:isaac@jago.community)

Once you have rust and its suite of tools installed, change the context of your terminal to this respository's directory:

```
cd ~/local/jago
```

If you want to learn about Vim and the basics of how to use it, type the following into your terminal and press enter:

```
vimtutor
```

If you need some inspiration for what to do with Vim, start by opening this exact file. Do this by typing the following in your terminal and pressing enter:

```
vim ~/local/jago/start
```

Now that you're inside the editor, let's configure it.

Open your configuration file by pressing `:` then typing the following and pressing enter:

```
e $MYVIMRC
```

This file describes to Vim how it should operate. It might be empty for you right now. I haven't started from scratch with Vim in quite a while. In any case, this is where you describe to Vim how it should operate. For example, adding the following would allow you to use your mouse inside Vim (in a comparatively limited manner):

```
set mouse=a
```

There is a perfect vimrc for you and your writing needs, but it might take a bit to get there. If you are curious what I use to deal with my work, I took it from

If you want to see what mine currently looks like, here it is:

```
source $HOME/local/jago/shelf/vim/handle.vim
```

To save the file, press escape and then type `:w` and press enter. To close Vim, type `:q` and press enter. To save and close Vim at the same time, type `:wq` and press enter.
