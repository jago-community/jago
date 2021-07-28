function! InstrumentHandle(...) abort
    if has('terminal')
        let cmd = 'vertical terminal'
    else
        throw "upgrade vim to version 8.1 or higher"
    endif

    let before = substitute(a:1, '%', expand('%'), '')
    let after = a:2

    let combined = before . ' -- ' . after

    execute cmd 'cargo run --features ' combined
endfunction

nmap <silent> <Leader>s :call InstrumentHandle("server", "serve")<CR>
nmap <silent> <Leader>t :call InstrumentHandle("instrument", "test %")<CR>
nmap <silent> <Leader>tt :call InstrumentHandle("instrument", "test % -- --nocapture")<CR>
nmap <silent> <Leader>T :call InstrumentHandle("instrument","test --workspace")<CR>
nmap <silent> <Leader>TT :call InstrumentHandle("instrument", "test --workspace -- --nocapture")<CR>
