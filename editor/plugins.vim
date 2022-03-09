call plug#begin()

Plug 'rust-lang/rust.vim'
Plug 'easymotion/vim-easymotion'
Plug 'preservim/nerdcommenter'
Plug 'tpope/vim-fugitive'
Plug 'challenger-deep-theme/vim'
Plug 'tpope/vim-eunuch'
Plug 'vim-airline/vim-airline'
Plug 'vim-airline/vim-airline-themes'

Plug 'kana/vim-textobj-user'
Plug 'preservim/vim-pencil'
Plug 'preservim/vim-lexical'
Plug 'preservim/vim-litecorrect'
Plug 'preservim/vim-textobj-quote'
Plug 'preservim/vim-textobj-sentence'

if has('nvim')

Plug 'neoclide/coc.nvim', { 'branch': 'release' }

else 

Plug 'prabirshrestha/vim-lsp'
Plug 'mattn/vim-lsp-settings'
Plug 'Shougo/ddc.vim'
Plug 'shun/ddc-vim-lsp'

endif

call plug#end()
