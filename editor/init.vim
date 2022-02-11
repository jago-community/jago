augroup vim_filetype
  au!
  autocmd BufNewFile,BufRead vim set syntax=vim
augroup END

augroup nvim_filetype
  au!
  autocmd BufNewFile,BufRead nvim set syntax=vim
augroup END

if has("nvim")
  source ~/jago/editor/nvim
else
  source ~/jago/editor/vim
endif
