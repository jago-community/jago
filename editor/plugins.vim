call plug#begin()

Plug 'rust-lang/rust.vim'
Plug 'easymotion/vim-easymotion'
Plug 'preservim/nerdcommenter'
Plug 'tpope/vim-fugitive'
Plug 'challenger-deep-theme/vim'
Plug 'tpope/vim-eunuch'

if has('nvim')

Plug 'neoclide/coc.nvim', { 'branch': 'release' }

else 

Plug 'prabirshrestha/vim-lsp'
Plug 'mattn/vim-lsp-settings'
Plug 'Shougo/ddc.vim'
Plug 'shun/ddc-vim-lsp'

endif

call plug#end()
