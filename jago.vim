set encoding=utf-8

set hidden

set cmdheight=2

set updatetime=300

set shortmess+=c

let mapleader = ","
let maplocalleader = ","

set noswapfile
set nobackup
set nowritebackup

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

" fail safe for a decaying world
nmap <silent> <Leader>w :w<CR>
nmap <silent> <Leader>q :q<CR>
nmap <silent> <Leader>wq :wq<CR>
nmap <silent> <Leader>e :Ex<CR>
nmap <silent> <Leader>r :source $MYVIMRC<CR>
nmap <silent> <Leader>bd :bd<CR>
nmap <silent> <Leader>gg :G<CR>
nmap <silent> <Leader>t :vert term<CR>

function Debug()
  let text = "const dbg = (x, tag) => { console.log(...['dbg!', tag, JSON.stringify(x, null, 2)].filter(y => !!y)); return x; };"

  exe "normal! a" . text . "\<Esc>"
endfunction

nmap <silent> <Leader>dbg :call Debug()<CR>

augroup filetypedetect
    au BufRead,BufNewFile *.nvim setfiletype vim
    au BufNewFile,BufRead *.nvim set syntax=vim
augroup END

nmap <silent> <Leader>e :Ex<CR>


""" BEGIN COC

" Use `[g` and `]g` to navigate diagnostics
" Use `:CocDiagnostics` to get all diagnostics of current buffer in location list.
nmap <silent> [g <Plug>(coc-diagnostic-prev)
nmap <silent> ]g <Plug>(coc-diagnostic-next)

" GoTo code navigation.
nmap <silent> gd <Plug>(coc-definition)
nmap <silent> gy <Plug>(coc-type-definition)
nmap <silent> gi <Plug>(coc-implementation)
nmap <silent> gr <Plug>(coc-references)

nmap <silent> mb :CocList buffers<CR>
nmap <silent> mf :CocList files<CR>
nmap <silent> <Leader>p :CocCommand prettier.formatFile<CR>


" Use K to show documentation in preview window.
nnoremap <silent> sj :call <SID>show_documentation()<CR>

function! s:show_documentation()
  if (index(['vim','help'], &filetype) >= 0)
    execute 'h '.expand('<cword>')
  elseif (coc#rpc#ready())
    call CocActionAsync('doHover')
  else
    execute '!' . &keywordprg . " " . expand('<cword>')
  endif
endfunction

" Highlight the symbol and its references when holding the cursor.
autocmd CursorHold * silent call CocActionAsync('highlight')

" Symbol renaming.
nmap <leader>rn <Plug>(coc-rename)

" Formatting selected code.
xmap <leader>f  <Plug>(coc-format-selected)
nmap <leader>f  <Plug>(coc-format-selected)

augroup mygroup
  autocmd!
  " Setup formatexpr specified filetype(s).
  autocmd FileType typescript,json setl formatexpr=CocAction('formatSelected')
  " Update signature help on jump placeholder.
  autocmd User CocJumpPlaceholder call CocActionAsync('showSignatureHelp')
augroup end

" Applying codeAction to the selected region.
" Example: `<leader>aap` for current paragraph
xmap <leader>a  <Plug>(coc-codeaction-selected)
nmap <leader>a  <Plug>(coc-codeaction-selected)

" Remap keys for applying codeAction to the current buffer.
nmap <leader>ac  <Plug>(coc-codeaction)
" Apply AutoFix to problem on the current line.
nmap <leader>qf  <Plug>(coc-fix-current)

" Map function and class text objects
" NOTE: Requires 'textDocument.documentSymbol' support from the language server.
xmap if <Plug>(coc-funcobj-i)
omap if <Plug>(coc-funcobj-i)
xmap af <Plug>(coc-funcobj-a)
omap af <Plug>(coc-funcobj-a)
xmap ic <Plug>(coc-classobj-i)
omap ic <Plug>(coc-classobj-i)
xmap ac <Plug>(coc-classobj-a)
omap ac <Plug>(coc-classobj-a)

" Remap <C-f> and <C-b> for scroll float windows/popups.
if has('nvim-0.4.0') || has('patch-8.2.0750')
  nnoremap <silent><nowait><expr> <C-f> coc#float#has_scroll() ? coc#float#scroll(1) : "\<C-f>"
  nnoremap <silent><nowait><expr> <C-b> coc#float#has_scroll() ? coc#float#scroll(0) : "\<C-b>"
  inoremap <silent><nowait><expr> <C-f> coc#float#has_scroll() ? "\<c-r>=coc#float#scroll(1)\<cr>" : "\<Right>"
  inoremap <silent><nowait><expr> <C-b> coc#float#has_scroll() ? "\<c-r>=coc#float#scroll(0)\<cr>" : "\<Left>"
  vnoremap <silent><nowait><expr> <C-f> coc#float#has_scroll() ? coc#float#scroll(1) : "\<C-f>"
  vnoremap <silent><nowait><expr> <C-b> coc#float#has_scroll() ? coc#float#scroll(0) : "\<C-b>"
endif

" Add `:Format` command to format current buffer.
command! -nargs=0 Format :call CocAction('format')

" Add `:Fold` command to fold current buffer.
command! -nargs=? Fold :call     CocAction('fold', <f-args>)

" Add `:OR` command for organize imports of the current buffer.
command! -nargs=0 OR   :call     CocAction('runCommand', 'editor.action.organizeImport')

" Add (Neo)Vim's native statusline support.
" NOTE: Please see `:h coc-status` for integrations with external plugins that
" provide custom statusline: lightline.vim, vim-airline.
set statusline^=%{coc#status()}%{get(b:,'coc_current_function','')}

if getcwd() ==# '$HOME/work'
    colorscheme meh
else
    colorscheme challenger_deep
endif

