function! InstrumentHandle(...) abort
    if has('terminal') || exists(':terminal')
        let cmd = 'vertical terminal'
    else
        throw "upgrade vim to version 8.1 or higher"
    endif

    let input = substitute(a:1, '%', expand('%'), '')

    execute cmd input
endfunction

nmap <silent> <Leader>c :call InstrumentHandle("cargo check --package author")<CR>

nmap <silent> <C-s> :call InstrumentHandle("./shelf/web/start")<CR>

autocmd BufNewFile,BufRead * if expand('%:t') !~ '\.' | set spell | endif
