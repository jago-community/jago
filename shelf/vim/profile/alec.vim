set hidden
syntax on
syntax enable

" enable project level configuratons
set exrc

" Enable file specific behavior like syntax highlighting and indentation
filetype on
filetype plugin on

let mapleader = ","
let maplocalleader = ","

let g:netwr_winsize = 20

" Magic cursor switching?
let &t_ti.="\e[1 q"
let &t_SI.="\e[5 q"
let &t_EI.="\e[1 q"
let &t_te.="\e[0 q"

filetype plugin on
let g:rustfmt_autosave = 1

" BEGIN config
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

" set cursorline
" set cursorcolumn

colorscheme challenger_deep
set laststatus=2

" Remember last position in file
au BufReadPost * if line("'\"") > 0 && line("'\"") <= line("$") | exe "normal g'\"" | endif

" Swap files
" set directory=$HOME/.vim/swp//
" set backupdir=$HOME/.vim/backup//

set undofile
set undodir=~/.vim/undo//
set noswapfile
set nobackup
set nowritebackup

" No arrow keys
map <up> <nop>
map <down> <nop>
map <left> <nop>
map <right> <nop>
imap <up> <nop>
imap <down> <nop>
imap <left> <nop>
imap <right> <nop>

" If we can, make undo history persistent.

" command history
set history=1000

set showcmd

" Increment number with ctrl+i
nmap <C-i> <C-a>

set backspace=indent,eol,start

" END config

"set complete+=kspell

" rust
"nmap <silent> <Leader>c :Cargo check<CR>
"nmap <silent> <Leader>t :Cargo test<CR>
"nmap <silent> <Leader>r :Cargo run<CR>

" lightline
let g:lightline = { 'colorscheme': 'challenger_deep' }

" ale
"let g:ale_fixers = {
"\   '*': ['remove_trailing_lines', 'trim_whitespace']
"\ }

"let g:ale_rust_cargo_default_feature_behavior = 'all'
"let g:ale_linters = {'rust': ['analyzer']}

"let g:ale_fix_on_save = 1
"let g:ale_completion_autoimport = 1
"let g:ale_completion_enabled = 1
"set omnifunc=ale#completion#OmniFunc

"nmap <silent> <Leader>gd :ALEGoToDefinition<CR>
"nmap <silent> <Leader>gt :ALEGoToTypeDefinition<CR>
"nmap <silent> <Leader>gr :ALEFindReferences<CR>
"nmap <silent> <Leader>gr :ALERename<CR>
"nmap <silent> <Leader>h :ALEHover<CR>
"nmap <silent> <Leader>h :ALEHover<CR>

" fzf
set wildignore+=*/tmp/*,*.so,*.swp,*.zip,*/node_modules/*

nmap <silent> <Leader>f :Files<CR>
nmap <silent> <Leader>b :Buffers<CR>

autocmd BufNewFile,BufRead * if expand('%:t') !~ '\.' | set spell | endif

if executable('rust-analyzer')
  au User lsp_setup call lsp#register_server({
        \   'name': 'Rust Language Server',
        \   'cmd': {server_info->['rust-analyzer']},
        \   'whitelist': ['rust'],
        \   'initialization_options': {
        \     'cargo': {
        \       'loadOutDirsFromCheck': v:true,
        \     },
        \     'procMacro': {
        \       'enable': v:true,
        \     },
        \   },
        \ })
endif

let g:lsp_diagnostics_echo_delay = 500
let g:lsp_diagnostics_enabled = 1
let g:lsp_diagnostics_echo_cursor = 1
let g:lsp_format_sync_timeout = 1000

function! s:on_lsp_buffer_enabled() abort
    setlocal omnifunc=lsp#complete
    setlocal signcolumn=yes
    nmap <buffer> <Leader>gd <plug>(lsp-definition)
    nmap <buffer> <Leader>gs <plug>(lsp-document-symbol-search)
    nmap <buffer> <Leader>gS <plug>(lsp-workspace-symbol-search)
    nmap <buffer> <Leader>gr <plug>(lsp-references)
    nmap <buffer> <Leader>gi <plug>(lsp-implementation)
    nmap <buffer> <Leader>gt <plug>(lsp-type-definition)
    nmap <buffer> <Leader>gn <plug>(lsp-rename)
    nmap <buffer> K <plug>(lsp-hover)
    inoremap <buffer> <expr><c-f> lsp#scroll(+4)
    inoremap <buffer> <expr><c-d> lsp#scroll(-4)

    autocmd! BufWritePre *.rs call execute('LspDocumentFormatSync')

    " refer to doc to add more commands
endfunction

augroup lsp_install
    au!
    " call s:on_lsp_buffer_enabled only for languages that has the server registered.
    autocmd User lsp_buffer_enabled call s:on_lsp_buffer_enabled()
augroup END
