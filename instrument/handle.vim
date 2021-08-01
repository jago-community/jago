function! InstrumentHandle(...) abort
    if has('terminal')
        let cmd = 'vertical terminal'
    else
        throw "upgrade vim to version 8.1 or higher"
    endif

    let before = substitute(a:1, '%', expand('%'), '')
    let after = a:2

    let combined = before . ' -- ' . after

    let action = 'cargo run --features ' . combined

    echo(action)

    execute cmd action
endfunction

nmap <silent> <Leader>s :call InstrumentHandle("context,server", "serve")<CR>

nmap <silent> <Leader>t :call InstrumentHandle("instrument", "test %")<CR>
nmap <silent> <Leader>tt :call InstrumentHandle("instrument", "test % -- --nocapture")<CR>
nmap <silent> <Leader>wt :call InstrumentHandle("instrument", "test --workspace")<CR>
nmap <silent> <Leader>wtt :call InstrumentHandle("instrument", "test --workspace -- --nocapture")<CR>

nmap <silent> <Leader>bu :call InstrumentHandle("instrument", "build %")<CR>
nmap <silent> <Leader>wbu :call InstrumentHandle("instrument", "build --workspace")<CR>

nmap <silent> <Leader>ch :call InstrumentHandle("instrument", "build %")<CR>
nmap <silent> <Leader>wch :call InstrumentHandle("instrument", "build --workspace")<CR>
