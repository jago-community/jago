function! InstrumentHandle(...) abort
    let action = substitute(a:1, '\s\+$', '', '')

    if has('terminal')
        let cmd = 'vertical terminal'
    else
        throw "upgrade vim to version 8.1 or higher"
    endif

    let input = []

    for index in range(2, a:0)
        call add(input, a:000[index-1])
    endfor

    let rest = len(input) > 0 ? ' ' . join(input, ' ') : ''

    let location = expand('%') . ' '

    let action = substitute(a:1, '%', location, '')

    let args = action . rest

    execute cmd 'cargo run --features instrument -- ' args
endfunction

nmap <silent> <Leader>t :call InstrumentHandle("test %")<CR>
nmap <silent> <Leader>tt :call InstrumentHandle("test % -- --nocapture")<CR>
nmap <silent> <Leader>T :call InstrumentHandle("test --workspace")<CR>
nmap <silent> <Leader>TT :call InstrumentHandle("test --workspace -- --nocapture")<CR>
