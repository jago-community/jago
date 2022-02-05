syntax enable
filetype plugin indent on

set noswapfile
set nobackup
set nowritebackup
set encoding=utf-8
set hidden
set cmdheight=2
set updatetime=300
set shortmess+=c
let mapleader = ","
let maplocalleader = ","
set autoread
set scrolloff=5
set sidescrolloff=5
set confirm
set encoding=utf-8
set wildmenu
set autoindent
filetype plugin indent on
set tabstop=4
set expandtab
set shiftwidth=4
set mouse=a
set number

" Search
set ignorecase		" Searching isn't case sensitive
set smartcase		" But when search contains uppercase it is case sensitive :)
set incsearch		" Highlight search results while typing
set hlsearch		" Highlight search results

" No arrow keys
map <up> <nop>
map <down> <nop>
map <left> <nop>
map <right> <nop>
imap <up> <nop>
imap <down> <nop>
imap <left> <nop>
imap <right> <nop>

set backspace=indent,eol,start
nmap <silent> <Leader>e :Ex<CR>

let g:rustfmt_autosave = 1

map <C-h> :LspHover<CR>
map <C-j> :LspDefinition<CR>
map <C-d> :LspDocumentDiagnostics<CR>
inoremap <C-@> <Esc>
